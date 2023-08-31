use crate::*;
use near_sdk::{is_promise_success, log, near_bindgen, Balance};

use crate::errors::*;
use crate::events::Event;
use crate::types::*;
use crate::utils::*;

const MIN_AMOUNT_TO_PERFORM_STAKE: Balance = ONE_NEAR;
const MIN_AMOUNT_TO_PERFORM_UNSTAKE: Balance = ONE_NEAR;
const MAX_SYNC_BALANCE_DIFF: Balance = 100;

/// Actions that should be called by off-chain actors
/// during each epoch.
#[near_bindgen]
impl LiquidStakingContract {
    // `stake_to_validator` and `unstake_from_validator` are used to mock
    // stake amounts and unstake amounts of validators at the beginnings
    // of simulation tests
    #[payable]
    #[cfg(feature = "test")]
    pub fn stake_to_validator(&mut self, validator_id: AccountId, amount: U128) {
        self.assert_running();
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_STAKE + GAS_EXT_DEPOSIT_AND_STAKE + GAS_CB_VALIDATOR_STAKED;

        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        Event::EpochStakeAttempt {
            validator_id: &validator_id,
            amount: &amount,
        }
        .emit();

        self.epoch_requested_stake_amount -= amount.0;

        // do staking on selected validator
        validator
            .deposit_and_stake(&mut self.validator_pool, amount.into())
            .then(ext_self_action_cb::validator_staked_callback(
                validator.account_id.clone(),
                amount.into(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_STAKED,
            ));
    }

    #[cfg(feature = "test")]
    pub fn unstake_from_validator(&mut self, validator_id: AccountId, amount: U128) {
        self.assert_running();
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_UNSTAKE + GAS_EXT_UNSTAKE + GAS_CB_VALIDATOR_UNSTAKED;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        Event::EpochUnstakeAttempt {
            validator_id: &validator_id,
            amount: &amount,
        }
        .emit();

        self.epoch_requested_unstake_amount -= amount.0;

        // do staking on selected validator
        validator
            .unstake(&mut self.validator_pool, amount.into())
            .then(ext_self_action_cb::validator_unstaked_callback(
                validator.account_id.clone(),
                amount.into(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_STAKED,
            ));
    }

    pub fn epoch_stake(&mut self) -> bool {
        self.assert_running();
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

        let candidate = self
            .validator_pool
            .get_candidate_to_stake(self.stake_amount_to_settle, self.total_staked_near_amount);

        if candidate.is_none() {
            log!("no candidate found to stake");
            return false;
        }

        let mut candidate = candidate.unwrap();
        let amount_to_stake = candidate.amount;

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
            validator_id: &candidate.validator.account_id,
            amount: &U128(amount_to_stake),
        }
        .emit();

        // do staking on selected validator
        candidate
            .validator
            .deposit_and_stake(&mut self.validator_pool, amount_to_stake)
            .then(ext_self_action_cb::validator_staked_callback(
                candidate.validator.account_id.clone(),
                amount_to_stake.into(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_STAKED,
            ));

        true
    }

    pub fn epoch_unstake(&mut self) -> bool {
        self.assert_running();
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

        let candidate = self.validator_pool.get_candidate_to_unstake_v2(
            self.unstake_amount_to_settle,
            self.total_staked_near_amount,
        );
        if candidate.is_none() {
            log!("no candidate found to unstake");
            return false;
        }
        let mut candidate = candidate.unwrap();
        let amount_to_unstake = candidate.amount;

        if amount_to_unstake < MIN_AMOUNT_TO_PERFORM_UNSTAKE {
            log!("unstake amount too low: {}", amount_to_unstake);
            return false;
        }

        // update internal state
        self.unstake_amount_to_settle -= amount_to_unstake;

        Event::EpochUnstakeAttempt {
            validator_id: &candidate.validator.account_id,
            amount: &U128(amount_to_unstake),
        }
        .emit();

        // do unstaking on selected validator
        candidate
            .validator
            .unstake(&mut self.validator_pool, amount_to_unstake)
            .then(ext_self_action_cb::validator_unstaked_callback(
                candidate.validator.account_id,
                amount_to_unstake.into(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_UNSTAKED,
            ));

        true
    }

    pub fn epoch_update_rewards(&mut self, validator_id: AccountId) {
        self.assert_running();

        let min_gas = GAS_EPOCH_UPDATE_REWARDS + GAS_EXT_GET_BALANCE + GAS_CB_VALIDATOR_GET_BALANCE;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        if validator.staked_amount == 0 && validator.unstaked_amount == 0 {
            return;
        }

        validator
            .refresh_total_balance(&mut self.validator_pool)
            .then(ext_self_action_cb::validator_get_balance_callback(
                validator.account_id,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_GET_BALANCE,
            ));
    }

    pub fn epoch_withdraw(&mut self, validator_id: AccountId) {
        self.assert_running();
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

        require!(!validator.draining, ERR_DRAINING);

        let amount = validator.unstaked_amount;

        Event::EpochWithdrawAttempt {
            validator_id: &validator_id,
            amount: &U128(amount),
        }
        .emit();

        validator.withdraw(&mut self.validator_pool, amount).then(
            ext_self_action_cb::validator_withdraw_callback(
                validator.account_id.clone(),
                amount.into(),
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

        Event::EpochCleanup {
            stake_amount_to_settle: &U128(self.stake_amount_to_settle),
            unstake_amount_to_settle: &U128(self.unstake_amount_to_settle),
        }
        .emit();
    }

    // To mock unsettled amounts at the beginnings of simulation tests
    #[cfg(feature = "test")]
    pub fn epoch_cleanup_for_test(&mut self) {
        self.epoch_cleanup();
    }

    /// Due to shares calculation and rounding of staking pool contract,
    /// the amount of staked and unstaked balance might be a little bit
    /// different than we requested.
    /// This method is to sync the actual numbers with the validator.
    pub fn sync_balance_from_validator(&mut self, validator_id: AccountId) {
        self.assert_running();

        let min_gas = GAS_SYNC_BALANCE + GAS_EXT_GET_ACCOUNT + GAS_CB_VALIDATOR_SYNC_BALANCE;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        validator
            .sync_account_balance(&mut self.validator_pool)
            .then(ext_self_action_cb::validator_get_account_callback(
                validator.account_id,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_SYNC_BALANCE,
            ));
    }
}

/// -- callbacks

#[ext_contract(ext_self_action_cb)]
trait EpochActionCallbacks {
    fn validator_staked_callback(&mut self, validator_id: AccountId, amount: U128);

    fn validator_unstaked_callback(&mut self, validator_id: AccountId, amount: U128);

    fn validator_get_balance_callback(&mut self, validator_id: AccountId);

    fn validator_get_account_callback(&mut self, validator_id: AccountId);

    fn validator_withdraw_callback(&mut self, validator_id: AccountId, amount: U128);
}

/// callbacks
/// functions here SHOULD NOT PANIC!
#[near_bindgen]
impl LiquidStakingContract {
    #[private]
    pub fn validator_staked_callback(&mut self, validator_id: AccountId, amount: U128) {
        let amount = amount.into();
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .unwrap_or_else(|| panic!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        if is_promise_success() {
            validator.on_stake_success(&mut self.validator_pool, amount);

            Event::EpochStakeSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        } else {
            validator.on_stake_failed(&mut self.validator_pool);

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
    pub fn validator_unstaked_callback(&mut self, validator_id: AccountId, amount: U128) {
        let amount = amount.into();
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .unwrap_or_else(|| panic!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

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
            validator.on_unstake_failed(&mut self.validator_pool);

            Event::EpochUnstakeFailed {
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

        if rewards == 0 {
            return;
        }

        self.total_staked_near_amount += rewards;
        self.internal_distribute_staking_rewards(rewards);
    }

    #[private]
    pub fn validator_get_account_callback(
        &mut self,
        validator_id: AccountId,
        #[callback] account: HumanReadableAccount,
    ) {
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .unwrap_or_else(|| panic!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        // allow at most MAX_SYNC_BALANCE_DIFF diff in total balance, staked balance and unstake balance
        let new_total_balance = account.staked_balance.0 + account.unstaked_balance.0;
        if abs_diff_eq(
            new_total_balance,
            validator.total_balance(),
            MAX_SYNC_BALANCE_DIFF,
        ) && abs_diff_eq(
            account.staked_balance.0,
            validator.staked_amount,
            MAX_SYNC_BALANCE_DIFF,
        ) && abs_diff_eq(
            account.unstaked_balance.0,
            validator.unstaked_amount,
            MAX_SYNC_BALANCE_DIFF,
        ) {
            Event::SyncValidatorBalanceSuccess {
                validator_id: &validator_id,
                old_staked_balance: &validator.staked_amount.into(),
                old_unstaked_balance: &validator.unstaked_amount.into(),
                old_total_balance: &validator.total_balance().into(),
                new_staked_balance: &account.staked_balance,
                new_unstaked_balance: &account.unstaked_balance,
                new_total_balance: &new_total_balance.into(),
            }
            .emit();
            validator.on_sync_account_balance_success(
                &mut self.validator_pool,
                account.staked_balance.0,
                account.unstaked_balance.0,
            );
        } else {
            Event::SyncValidatorBalanceFailed {
                validator_id: &validator_id,
                old_staked_balance: &validator.staked_amount.into(),
                old_unstaked_balance: &validator.unstaked_amount.into(),
                old_total_balance: &validator.total_balance().into(),
                new_staked_balance: &account.staked_balance,
                new_unstaked_balance: &account.unstaked_balance,
                new_total_balance: &new_total_balance.into(),
            }
            .emit();
            validator.on_sync_account_balance_failed(&mut self.validator_pool);
        }
    }

    #[private]
    pub fn validator_withdraw_callback(&mut self, validator_id: AccountId, amount: U128) {
        let amount = amount.into();
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .unwrap_or_else(|| panic!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        if is_promise_success() {
            validator.on_withdraw_success(&mut self.validator_pool);

            Event::EpochWithdrawSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        } else {
            // withdraw failed, revert
            validator.on_withdraw_failed(&mut self.validator_pool, amount);

            Event::EpochWithdrawFailed {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        }
    }
}
