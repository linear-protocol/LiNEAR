use crate::*;
use near_sdk::{log, near_bindgen, Balance, is_promise_success,};

use crate::errors::*;
use crate::events::Event;
use crate::types::*;
use crate::utils::*;

const MIN_AMOUNT_TO_PERFORM_STAKE: Balance = ONE_NEAR;
const MIN_AMOUNT_TO_PERFORM_UNSTAKE: Balance = ONE_NEAR;

/// Actions that should be called by off-chain actors
/// during each epoch.
#[near_bindgen]
impl LiquidStakingContract {
    pub fn epoch_stake(&mut self) -> bool {
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_STAKE + GAS_EXT_DEPOSIT_AND_STAKE + GAS_CB_VALIDATOR_STAKED;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        self.epoch_cleanup();
        // after cleanup, there might be no need to stake
        if self.stake_amount_to_settle == 0 {
            log!("no need to stake, amount to settle is zero");
            return false;
        }

        let (candidate, amount_to_stake) = self
            .validator_pool
            .get_candidate_to_stake(self.stake_amount_to_settle, self.total_staked_near_amount);

        if candidate.is_none() {
            log!("no candidate found to stake {}", amount_to_stake);
            return false;
        }
        let mut candidate = candidate.unwrap();

        if amount_to_stake < MIN_AMOUNT_TO_PERFORM_STAKE {
            log!("stake amount too low: {}", amount_to_stake);
            return false;
        }

        require!(
            env::account_balance() >= amount_to_stake + CONTRACT_MIN_RESERVE_BALANCE,
            ERR_MIN_RESERVE
        );

        // update internal state
        self.stake_amount_to_settle -= amount_to_stake;

        Event::EpochStakeAttempt {
            validator_id: &candidate.account_id,
            amount: &U128(amount_to_stake),
        }
        .emit();

        // do staking on selected validator
        candidate.deposit_and_stake(amount_to_stake).then(
            ext_self_action_cb::validator_staked_callback(
                candidate.account_id.clone(),
                amount_to_stake,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_STAKED,
            ),
        );

        return true;
    }

    pub fn epoch_unstake(&mut self) -> bool {
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_UNSTAKE + GAS_EXT_UNSTAKE + GAS_CB_VALIDATOR_UNSTAKED;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        self.epoch_cleanup();
        // after cleanup, there might be no need to unstake
        if self.unstake_amount_to_settle == 0 {
            log!("no need to unstake, amount to settle is zero");
            return false;
        }

        let (candidate, amount_to_unstake) = self
            .validator_pool
            .get_candidate_to_unstake(self.unstake_amount_to_settle, self.total_staked_near_amount);
        if candidate.is_none() {
            log!("no candidate found to unstake {}", amount_to_unstake);
            return false;
        }
        let mut candidate = candidate.unwrap();

        if amount_to_unstake < MIN_AMOUNT_TO_PERFORM_UNSTAKE {
            log!("unstake amount too low: {}", amount_to_unstake);
            return false;
        }

        // update internal state
        self.unstake_amount_to_settle -= amount_to_unstake;

        Event::EpochUnstakeAttempt {
            validator_id: &candidate.account_id,
            amount: &U128(amount_to_unstake),
        }
        .emit();

        // do unstaking on selected validator
        candidate
            .unstake(&mut self.validator_pool, amount_to_unstake)
            .then(ext_self_action_cb::validator_unstaked_callback(
                candidate.account_id,
                amount_to_unstake,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_UNSTAKED,
            ));

        return true;
    }

    pub fn epoch_update_rewards(&mut self, validator_id: AccountId) {
        let min_gas = GAS_EPOCH_UPDATE_REWARDS + GAS_EXT_GET_BALANCE + GAS_CB_VALIDATOR_GET_BALANCE;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        if validator.staked_amount == 0 && validator.unstaked_amount == 0 {
            return;
        }

        validator
            .refresh_total_balance()
            .then(ext_self_action_cb::validator_get_balance_callback(
                validator.account_id,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_GET_BALANCE,
            ));
    }

    pub fn epoch_withdraw(&mut self, validator_id: AccountId) {
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_WITHDRAW + GAS_EXT_WITHDRAW + GAS_CB_VALIDATOR_WITHDRAW;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        let amount = validator.unstaked_amount;

        Event::EpochWithdrawAttempt {
            validator_id: &validator_id,
            amount: &U128(amount),
        }
        .emit();

        validator.withdraw(&mut self.validator_pool, amount).then(
            ext_self_action_cb::validator_withdraw_callback(
                validator.account_id.clone(),
                amount,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_WITHDRAW,
            ),
        );
    }

    /// Cleaning up stake requirements and unstake requirements,
    /// since some stake requirements could be eliminated if
    /// there are more unstake requirements, and vice versa.
    fn epoch_cleanup(&mut self) {
        if self.last_settlement_epoch == get_epoch_height() {
            return;
        }
        self.last_settlement_epoch = get_epoch_height();

        // here we use += because cleanup amount might not be 0
        self.stake_amount_to_settle += self.epoch_requested_stake_amount;
        self.unstake_amount_to_settle += self.epoch_requested_unstake_amount;
        self.epoch_requested_stake_amount = 0;
        self.epoch_requested_unstake_amount = 0;

        if self.stake_amount_to_settle > self.unstake_amount_to_settle {
            self.stake_amount_to_settle -= self.unstake_amount_to_settle;
            self.unstake_amount_to_settle = 0;
        } else {
            self.unstake_amount_to_settle -= self.stake_amount_to_settle;
            self.stake_amount_to_settle = 0;
        }
    }

    /// Due to shares calculation and rounding of staking pool contract,
    /// the amount of staked and unstaked balance might be a little bit
    /// different than we requested.
    /// This method is to sync the actual numbers with the validator.
    pub fn sync_account_balance(&mut self, validator_id: AccountId) {
        let min_gas = GAS_SYNC_BALANCE + GAS_EXT_GET_ACCOUNT + GAS_CB_VALIDATOR_SYNC_BALANCE;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        validator
            .sync_balance_from_validator()
            .then(ext_self_action_cb::validator_get_account_callback(
                validator.account_id,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_SYNC_BALANCE
            ));
    }

    /// This method is designed to drain a validator.
    /// The weight of target validator should be set to 0 before calling this.
    /// And a following call to drain_withdraw MUST be made after 4 epoches.
    pub fn drain_unstake(&mut self, validator_id: AccountId) {
        self.assert_manager();

        // make sure enough gas was given
        let min_gas = GAS_DRAIN_UNSTAKE + GAS_EXT_UNSTAKE + GAS_CB_VALIDATOR_UNSTAKED;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self.validator_pool.get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        // make sure the validator:
        // 1. has weight set to 0
        // 2. not in pending release
        // 3. has not unstaked balance (because this part is from user's unstake request)
        require!(
            validator.weight == 0,
            ERR_NON_ZERO_WEIGHT
        );
        require!(
            !validator.pending_release(),
            ERR_VALIDATOR_UNSTAKE_WHEN_LOCKED
        );
        require!(
            validator.unstaked_amount == 0,
            ERR_NON_ZERO_UNSTAKED_AMOUNT
        );

        let unstake_amount = validator.staked_amount;

        Event::DrainUnstakeAttempt {
            validator_id: &validator_id,
            amount: &U128(unstake_amount)
        }
        .emit();

        // perform actual unstake
        validator
            .unstake(&mut self.validator_pool, unstake_amount)
            .then(ext_self_action_cb::validator_drain_unstaked_callback(
                validator.account_id,
                unstake_amount,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_UNSTAKED,
            ));
    }

    /// Withdraw from a drained validator
    pub fn drain_withdraw(&mut self, validator_id: AccountId) {
        self.assert_manager();

        // make sure enough gas was given
        let min_gas = GAS_DRAIN_WITHDRAW + GAS_EXT_WITHDRAW + GAS_CB_VALIDATOR_WITHDRAW;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        // make sure the validator:
        // 1. has weight set to 0
        // 2. has no staked balance
        // 3. not pending release
        require!(
            validator.weight == 0,
            ERR_NON_ZERO_WEIGHT
        );
        require!(
            validator.staked_amount == 0,
            ERR_NON_ZERO_STAKED_AMOUNT
        );
        require!(
            !validator.pending_release(),
            ERR_VALIDATOR_WITHDRAW_WHEN_LOCKED
        );

        let amount = validator.unstaked_amount;

        Event::DrainWithdrawAttempt {
            validator_id: &validator_id,
            amount: &U128(amount)
        }
        .emit();

        validator.withdraw(&mut self.validator_pool, amount).then(
            ext_self_action_cb::validator_drain_withdraw_callback(
                validator.account_id.clone(),
                amount,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_WITHDRAW,
            ),
        );
    }
}

/// -- callbacks

#[ext_contract(ext_self_action_cb)]
trait EpochActionCallbacks {
    fn validator_staked_callback(&mut self, validator_id: AccountId, amount: Balance);

    fn validator_unstaked_callback(&mut self, validator_id: AccountId, amount: Balance);

    fn validator_drain_unstaked_callback(&mut self, validator_id: AccountId, amount: Balance);

    fn validator_get_balance_callback(&mut self, validator_id: AccountId);

    fn validator_get_account_callback(&mut self, validator_id: AccountId);

    fn validator_withdraw_callback(&mut self, validator_id: AccountId, amount: Balance);

    fn validator_drain_withdraw_callback(&mut self, validator_id: AccountId, amount: Balance);
}

/// callbacks
/// functions here SHOULD NOT PANIC!
#[near_bindgen]
impl LiquidStakingContract {
    #[private]
    pub fn validator_staked_callback(&mut self, validator_id: AccountId, amount: Balance) {
        if is_promise_success() {
            let mut validator = self
                .validator_pool
                .get_validator(&validator_id)
                .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));
            validator.on_stake_success(&mut self.validator_pool, amount);

            Event::EpochStakeSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        } else {
            // stake failed, revert
            self.stake_amount_to_settle += amount;

            Event::EpochStakeFailed {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        }
    }

    #[private]
    pub fn validator_unstaked_callback(&mut self, validator_id: AccountId, amount: Balance) {
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        if is_promise_success() {
            validator.on_unstake_success(&mut self.validator_pool, amount);

            Event::EpochUnstakeSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        } else {
            // unstake failed, revert
            // 1. revert contract states
            self.unstake_amount_to_settle += amount;

            // 2. revert validator states
            validator.on_unstake_failed(&mut self.validator_pool, amount);

            Event::EpochUnstakeFailed {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        }
    }

    #[private]
    pub fn validator_drain_unstaked_callback(&mut self, validator_id: AccountId, amount: Balance) {
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        if is_promise_success() {
            validator.on_unstake_success(&mut self.validator_pool, amount);

            Event::DrainUnstakeSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        } else {
            // unstake failed, revert
            validator.on_unstake_failed(&mut self.validator_pool, amount);

            Event::DrainUnstakeFailed {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        }
    }

    #[private]
    pub fn validator_get_balance_callback(
        &mut self,
        validator_id: AccountId,
        #[callback] total_balance: U128,
    ) {
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        let new_balance = total_balance.0;
        let rewards = new_balance - validator.total_balance();
        Event::EpochUpdateRewards {
            validator_id: &validator_id,
            old_balance: &U128(validator.total_balance()),
            new_balance: &U128(new_balance),
            rewards: &U128(rewards),
        }
        .emit();

        validator.on_new_total_balance(&mut self.validator_pool, new_balance);

        if rewards <= 0 {
            return;
        }

        self.total_staked_near_amount += rewards;
        self.internal_distribute_staking_rewards(rewards);
    }

    #[private]
    pub fn validator_get_account_callback(
        &mut self, 
        validator_id: AccountId,
        #[callback] account: HumanReadableAccount
    ) {
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        validator.on_sync_balance_from_validator(
            &mut self.validator_pool, 
            account.staked_balance.0,
            account.unstaked_balance.0
        );

        Event::AccountBalanceSynced {
            validator_id: &validator_id,
            staked_balance: &account.staked_balance,
            unstaked_balance: &account.unstaked_balance
        }
        .emit();
    }

    #[private]
    pub fn validator_withdraw_callback(&mut self, validator_id: AccountId, amount: Balance) {
        if is_promise_success() {
            Event::EpochWithdrawSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
            return;
        }

        // withdraw failed, revert
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        validator.on_withdraw_failed(&mut self.validator_pool, amount);

        Event::EpochWithdrawFailed {
            validator_id: &validator_id,
            amount: &U128(amount),
        }
        .emit();
    }

    #[private]
    pub fn validator_drain_withdraw_callback(&mut self, validator_id: AccountId, amount: Balance) {
        if is_promise_success() {
            Event::DrainWithdrawSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
            
            // those funds need to be restaked, so we add them back to epoch request
            self.epoch_requested_stake_amount += amount;

            return;
        } else {
            // withdraw failed, revert
            let mut validator = self
                .validator_pool
                .get_validator(&validator_id)
                .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

            validator.on_withdraw_failed(&mut self.validator_pool, amount);

            Event::DrainWithdrawFailed {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        }
    }
}
