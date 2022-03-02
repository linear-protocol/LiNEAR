use crate::*;
use near_sdk::{
    near_bindgen, Balance, log, is_promise_success,
};

use crate::errors::*;
use crate::types::*;
use crate::utils::*;
use crate::events::*;

const MIN_AMOUNT_TO_PERFORM_STAKE: Balance = ONE_NEAR;
const MIN_AMOUNT_TO_PERFORM_UNSTAKE: Balance = ONE_NEAR;
/// min NEAR balance this contract should hold in order to cover
/// storage and contract call fees.
const CONTRACT_MIN_RESERVE_BALANCE: Balance = ONE_NEAR;

/// Actions that should be called by off-chain actors
/// during each epoch.
#[near_bindgen]
impl LiquidStakingContract {
    pub fn epoch_stake(&mut self) -> bool {
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_STAKE + GAS_EXT_DEPOSIT_AND_STAKE + GAS_CB_VALIDATOR_STAKED;
        require!(
            env::prepaid_gas() >= min_gas,
            format!(
                "{}. require at least {:?}", 
                ERR_NO_ENOUGH_GAS, 
                min_gas
            )
        );

        self.epoch_cleanup();
        // after cleanup, there might be no need to stake
        if self.epoch_requested_stake_amount == 0 {
            return false;
        }

        let (candidate, amount_to_stake) = self
            .validator_pool
            .get_candidate_to_stake(self.epoch_requested_stake_amount, self.total_staked_near_amount);

        if candidate.is_none() {
            // TODO no candidate found
            return false;
        }
        let mut candidate = candidate.unwrap();

        // DEBUG
        log!(
            "amount need stake: {}, candidate: {}, amount to stake: {}, candidate staked: {}",
            self.epoch_requested_stake_amount,
            candidate.account_id,
            amount_to_stake,
            candidate.staked_amount
        );

        if amount_to_stake < MIN_AMOUNT_TO_PERFORM_STAKE {
            log!("stake amount too low: {}", amount_to_stake);
            return false;
        }

        require!(
            env::account_balance() >= amount_to_stake + CONTRACT_MIN_RESERVE_BALANCE,
            ERR_MIN_RESERVE
        );

        // update internal state
        self.epoch_requested_stake_amount -= amount_to_stake;

        log_stake_attempt(&candidate.account_id, amount_to_stake);

        // do staking on selected validator
        candidate
            .deposit_and_stake(&mut self.validator_pool, amount_to_stake)
            .then(ext_self_action_cb::validator_staked_callback(
                candidate.account_id.clone(),
                amount_to_stake,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_STAKED
            ));

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
        if self.epoch_requested_unstake_amount == 0 {
            return false;
        }

        let (candidate, amount_to_unstake) = self
            .validator_pool
            .get_candidate_to_unstake(self.epoch_requested_unstake_amount, self.total_staked_near_amount);
        if candidate.is_none() {
            // TODO
            return false;
        }
        let mut candidate = candidate.unwrap();

        if amount_to_unstake < MIN_AMOUNT_TO_PERFORM_UNSTAKE {
            log!("unstake amount too low: {}", amount_to_unstake);
            return false;
        }

        // update internal state
        self.epoch_requested_unstake_amount -= amount_to_unstake;

        log_unstake_attempt(&candidate.account_id, amount_to_unstake);

        // do unstaking on selected validator
        candidate
            .unstake(&mut self.validator_pool, amount_to_unstake)
            .then(ext_self_action_cb::validator_unstaked_callback(
                candidate.account_id,
                amount_to_unstake,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_UNSTAKED
            ));

        return true;
    }

    pub fn epoch_update_rewards(
        &mut self,
        validator_id: AccountId,
    ) {
        let min_gas = GAS_EPOCH_UPDATE_REWARDS + GAS_EXT_GET_BALANCE + GAS_CB_VALIDATOR_GET_BALANCE;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let validator = self.validator_pool
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
                GAS_CB_VALIDATOR_GET_BALANCE
            ));
    }

    pub fn epoch_withdraw(&mut self, validator_id: AccountId) {
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_WITHDRAW + GAS_EXT_WITHDRAW + GAS_CB_VALIDATOR_WITHDRAW;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        let mut validator = self.validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        let amount = validator.unstaked_amount;

        log_withdraw_attempt(
            &validator_id,
            amount
        );

        validator
            .withdraw(&mut self.validator_pool, amount)
            .then(ext_self_action_cb::validator_withdraw_callback(
                validator.account_id.clone(),
                amount,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_WITHDRAW
            ));
    }

    /// Cleaning up stake requirements and unstake requirements,
    /// since some stake requirements could be eliminated if 
    /// there are more unstake requirements, and vice versa.
    fn epoch_cleanup(&mut self) {
        if self.epoch_requested_stake_amount > self.epoch_requested_unstake_amount {
            self.epoch_requested_stake_amount -= self.epoch_requested_unstake_amount;
            self.epoch_requested_unstake_amount = 0;
        } else {
            self.epoch_requested_unstake_amount -= self.epoch_requested_stake_amount;
            self.epoch_requested_stake_amount = 0;
        }
    }
}

/// -- callbacks

#[ext_contract(ext_self_action_cb)]
trait EpochActionCallbacks {
    fn validator_staked_callback(
        &mut self,
        validator_id: AccountId,
        amount: Balance
    );

    fn validator_unstaked_callback(
        &mut self,
        validator_id: AccountId,
        amount: Balance
    );

    fn validator_get_balance_callback(
        &mut self,
        validator_id: AccountId
    );

    fn validator_withdraw_callback(
        &mut self,
        validator_id: AccountId,
        amount: Balance
    );
}

/// callbacks
/// functions here SHOULD NOT PANIC!
#[near_bindgen]
impl LiquidStakingContract {
    pub fn validator_staked_callback(
        &mut self,
        validator_id: AccountId,
        amount: Balance
    ) {
        assert_is_callback();

        if is_promise_success() {
            log_stake_success(&validator_id, amount);
            return;
        }

        // stake failed, revert
        // 1. revert contract states
        self.epoch_requested_stake_amount += amount;

        // 2. revert validator states
        let mut validator = self.validator_pool
            .get_validator(&validator_id)
            .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        validator.on_stake_failed(&mut self.validator_pool, amount); 

        log_stake_failed(&validator_id, amount);
    }

    pub fn validator_unstaked_callback(
        &mut self,
        validator_id: AccountId,
        amount: Balance
    ) {
        assert_is_callback();

        if is_promise_success() {
            log_unstake_success(&validator_id, amount);
            return;
        }

        // unstake failed, revert
        // 1. revert contract states
        self.epoch_requested_unstake_amount += amount;

        // 2. revert validator states
        let mut validator = self.validator_pool
            .get_validator(&validator_id)
            .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        validator.on_unstake_failed(&mut self.validator_pool, amount);

        log_unstake_failed(&validator_id, amount);
    }

    pub fn validator_get_balance_callback(
        &mut self,
        validator_id: AccountId,
        #[callback] total_balance: U128 
    ) {
        assert_is_callback();

        let mut validator = self.validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        let new_balance = total_balance.0;
        let rewards = new_balance - validator.total_balance();
        log_new_balance(
            &validator_id,
            validator.total_balance(),
            new_balance,
            rewards
        );

        validator.on_new_total_balance(&mut self.validator_pool, new_balance);

        // TODO could reward < 0?
        if rewards <= 0 {
            return;
        }

        self.total_staked_near_amount += rewards;
        self.internal_distribute_rewards(rewards);
    }

    pub fn validator_withdraw_callback(
        &mut self,
        validator_id: AccountId,
        amount: Balance
    ) {
        assert_is_callback();

        if is_promise_success() {
            log_withdraw_success(&validator_id, amount);
            return;
        }

        // withdraw failed, revert
        let mut validator = self.validator_pool
            .get_validator(&validator_id)
            .expect(&format!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        validator.on_withdraw_failed(&mut self.validator_pool, amount);

        log_withdraw_failed(
            &validator_id,
            amount
        );
    }
}
