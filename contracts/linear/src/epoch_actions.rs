use crate::*;
use near_sdk::{is_promise_success, log, near_bindgen, Balance, PromiseError, PromiseOrValue};

use crate::errors::*;
use crate::events::Event;
use crate::types::*;
use crate::utils::*;

const MIN_AMOUNT_TO_PERFORM_STAKE: Balance = ONE_NEAR;
const MAX_SYNC_BALANCE_DIFF: Balance = 100;
const MANAGER_SYNC_BALANCE_DIFF_THRESHOLD: Balance = 1_000_000;

/// Actions that should be called by off-chain actors
/// during each epoch.
#[near_bindgen]
impl LiquidStakingContract {
    /// Stake $NEAR to one of the validators.
    ///
    /// Select a candidate validator and stake part of or all of the to-settle
    /// stake amounts to this validator. This function is expected to be called
    /// in each epoch.
    ///
    /// # Return
    /// * `true` - a candidate validator is selected and successfully staked to.
    ///            There might be more stake amounts to settle so this function
    ///            should be called again.
    /// * `false` - There is no need to call this function again in this epoch.
    pub fn epoch_stake(&mut self) -> PromiseOrValue<bool> {
        self.assert_running();
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_STAKE
            + GAS_EXT_DEPOSIT_AND_STAKE
            + GAS_CB_VALIDATOR_STAKED
            + GAS_SYNC_BALANCE
            + GAS_CB_VALIDATOR_SYNC_BALANCE;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        self.epoch_cleanup();
        // after cleanup, there might be no need to stake
        if self.stake_amount_to_settle == 0 {
            log!("no need to stake, amount to settle is zero");
            return PromiseOrValue::Value(false);
        }

        let candidate = self
            .validator_pool
            .get_candidate_to_stake(self.stake_amount_to_settle, self.total_staked_near_amount);

        if candidate.is_none() {
            log!("no candidate found to stake");
            return PromiseOrValue::Value(false);
        }

        let mut candidate = candidate.unwrap();
        let amount_to_stake = candidate.amount;

        if amount_to_stake < MIN_AMOUNT_TO_PERFORM_STAKE {
            log!("stake amount too low: {}", amount_to_stake);
            return PromiseOrValue::Value(false);
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
                GAS_CB_VALIDATOR_STAKED + GAS_SYNC_BALANCE + GAS_CB_VALIDATOR_SYNC_BALANCE,
            ))
            .into()
    }

    /// Unstake $NEAR from one of the validators.
    ///
    /// Select a candidate validator and unstake part of or all of the to-settle
    /// unstake amounts from this validator. This function is expected to be called
    /// in each epoch.
    ///
    /// # Return
    /// * `true` - a candidate validator is selected and successfully unstaked from.
    ///            There might be more unstake amounts to settle so this function
    ///            should be called again.
    /// * `false` - There is no need to call this function again in this epoch.
    pub fn epoch_unstake(&mut self) -> PromiseOrValue<bool> {
        self.assert_running();
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_UNSTAKE
            + GAS_EXT_UNSTAKE
            + GAS_CB_VALIDATOR_UNSTAKED
            + GAS_SYNC_BALANCE
            + GAS_CB_VALIDATOR_SYNC_BALANCE;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        self.epoch_cleanup();
        // after cleanup, there might be no need to unstake
        if self.unstake_amount_to_settle == 0 {
            log!("no need to unstake, amount to settle is zero");
            return PromiseOrValue::Value(false);
        }

        let candidate = self.validator_pool.get_candidate_to_unstake_v2(
            self.unstake_amount_to_settle,
            self.total_staked_near_amount,
        );
        if candidate.is_none() {
            log!("no candidate found to unstake");
            return PromiseOrValue::Value(false);
        }
        let mut candidate = candidate.unwrap();
        let amount_to_unstake = candidate.amount;

        // Since it's reasonable to unstake any amount of NEAR from a validator, as low as 1 yocto NEAR,
        // when its target stake amount is 0, here we don't enforce the minimun unstake amount requirement.

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
                GAS_CB_VALIDATOR_UNSTAKED + GAS_SYNC_BALANCE + GAS_CB_VALIDATOR_SYNC_BALANCE,
            ))
            .into()
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
}

/// -- callbacks

#[ext_contract(ext_self_action_cb)]
trait EpochActionCallbacks {
    fn validator_staked_callback(
        &mut self,
        validator_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<bool>;

    fn validator_unstaked_callback(
        &mut self,
        validator_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<bool>;

    fn validator_get_balance_callback(&mut self, validator_id: AccountId);

    fn validator_get_account_callback(
        &mut self,
        validator_id: AccountId,
        post_action: bool,
    ) -> bool;

    fn validator_withdraw_callback(&mut self, validator_id: AccountId, amount: U128);
}

/// callbacks
/// functions here SHOULD NOT PANIC!
#[near_bindgen]
impl LiquidStakingContract {
    /// # Return
    /// * `true` - Stake and sync balance succeed
    /// * `false` - Stake fails
    #[private]
    pub fn validator_staked_callback(
        &mut self,
        validator_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<bool> {
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

            validator
                .sync_account_balance(&mut self.validator_pool, true)
                .then(ext_self_action_cb::validator_get_account_callback(
                    validator_id,
                    true,
                    env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_CB_VALIDATOR_SYNC_BALANCE,
                ))
                .into()
        } else {
            validator.on_stake_failed(&mut self.validator_pool);

            // stake failed, revert
            self.stake_amount_to_settle += amount;

            Event::EpochStakeFailed {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();

            PromiseOrValue::Value(false)
        }
    }

    /// # Return
    /// * `true` - Unstake and sync balance succeed
    /// * `false` - Unstake fails
    #[private]
    pub fn validator_unstaked_callback(
        &mut self,
        validator_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<bool> {
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

            validator
                .sync_account_balance(&mut self.validator_pool, true)
                .then(ext_self_action_cb::validator_get_account_callback(
                    validator_id,
                    true,
                    env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_CB_VALIDATOR_SYNC_BALANCE,
                ))
                .into()
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

            PromiseOrValue::Value(false)
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

    /// Callback after get LiNEAR contract account balance from the validator
    ///
    /// Params:
    /// - validator_id: the validator to sync balance
    /// - post_action: sync balance is called after stake or unstake
    #[private]
    pub fn validator_get_account_callback(
        &mut self,
        validator_id: AccountId,
        post_action: bool,
        #[callback_result] result: Result<HumanReadableAccount, PromiseError>,
    ) -> bool {
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .unwrap_or_else(|| panic!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        let max_sync_balance_diff = if !post_action && self.signed_by_manager() {
            MANAGER_SYNC_BALANCE_DIFF_THRESHOLD
        } else {
            MAX_SYNC_BALANCE_DIFF
        };

        match result {
            Ok(account) => {
                // allow at most max_sync_balance_diff diff in total balance, staked balance and unstake balance
                let new_total_balance = account.staked_balance.0 + account.unstaked_balance.0;
                if abs_diff_eq(
                    new_total_balance,
                    validator.total_balance(),
                    max_sync_balance_diff,
                ) && abs_diff_eq(
                    account.staked_balance.0,
                    validator.staked_amount,
                    max_sync_balance_diff,
                ) && abs_diff_eq(
                    account.unstaked_balance.0,
                    validator.unstaked_amount,
                    max_sync_balance_diff,
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
                    Event::SyncValidatorBalanceFailedLargeDiff {
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
            Err(_) => {
                Event::SyncValidatorBalanceFailedCannotGetAccount {
                    validator_id: &validator_id,
                    old_staked_balance: &validator.staked_amount.into(),
                    old_unstaked_balance: &validator.unstaked_amount.into(),
                    old_total_balance: &validator.total_balance().into(),
                }
                .emit();
                validator.on_sync_account_balance_failed(&mut self.validator_pool);
            }
        };
        true
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
