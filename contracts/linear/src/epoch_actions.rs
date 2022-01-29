use crate::*;
use near_sdk::{
    near_bindgen, Balance, log, is_promise_success,
};

use crate::errors::*;
use crate::types::*;
use crate::utils::*;
use crate::events::*;

const MIN_AMOUNT_TO_PERFORM_STAKE: Balance = 10 * ONE_NEAR;
const MIN_AMOUNT_TO_PERFORM_UNSTAKE: Balance = 10 * ONE_NEAR;
/// min NEAR balance this contract should hold in order to cover
/// storage and contract call fees.
const CONTRACT_MIN_RESERVE_BALANCE: Balance = 30 * ONE_NEAR;

/// Actions that should be called by off-chain actors
/// during each epoch.
#[near_bindgen]
impl LiquidStakingContract {
    pub fn epoch_stake(&mut self) {
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_STAKE + GAS_EXT_DEPOSIT_AND_STAKE + GAS_CB_VALIDATOR_STAKED;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        self.epoch_cleanup();
        // after cleanup, there might be no need to stake
        if self.epoch_requested_stake_amount == 0 {
            return;
        }

        let (mut candidate, amount_to_stake) = self
            .validator_pool
            .get_candidate_to_stake(self.epoch_requested_stake_amount);

        if amount_to_stake < MIN_AMOUNT_TO_PERFORM_STAKE {
            log!(format!("stake amount too low: {}", amount_to_stake));
            return;
        }

        require!(
            env::account_balance() - CONTRACT_MIN_RESERVE_BALANCE >= amount_to_stake,
            ERR_MIN_RESERVE
        );

        // update internal state
        self.epoch_requested_stake_amount -= amount_to_stake;

        log_stake_attempt(&candidate.account_id, amount_to_stake);

        // do staking on selected validator
        candidate
            .deposit_and_stake(amount_to_stake)
            .then(ext_self_action_cb::validator_staked_callback(
                candidate.account_id,
                amount_to_stake,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_STAKED
            ));
    }

    pub fn epoch_unstake(&mut self) {
        // make sure enough gas was given
        let min_gas = GAS_EPOCH_UNSTAKE + GAS_EXT_UNSTAKE + GAS_CB_VALIDATOR_UNSTAKED;
        require!(
            env::prepaid_gas() >= min_gas,
            format!("{}. require at least {:?}", ERR_NO_ENOUGH_GAS, min_gas)
        );

        self.epoch_cleanup();
        // after cleanup, there might be no need to unstake
        if self.epoch_requested_unstake_amount == 0 {
            return;
        }

        let (mut candidate, amount_to_unstake) = self
            .validator_pool
            .get_candidate_to_unstake(self.epoch_requested_unstake_amount);

        if amount_to_unstake < MIN_AMOUNT_TO_PERFORM_UNSTAKE {
            log!(format!("unstake amount too low: {}", amount_to_unstake));
            return;
        }

        // update internal state
        self.epoch_requested_unstake_amount -= amount_to_unstake;

        log_unstake_attempt(&candidate.account_id, amount_to_unstake);

        // do unstaking on selected validator
        candidate
            .unstake(amount_to_unstake)
            .then(ext_self_action_cb::validator_unstaked_callback(
                candidate.account_id,
                amount_to_unstake,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_UNSTAKED
            ));
    }

    pub fn epoch_update_rewards(&mut self) {

    }

    pub fn epoch_withdraw(&mut self) {

    }

    /// Cleaning up stake requirements and unstake requirements,
    /// since some stake requirements could be eliminated if 
    /// there are more unstake requirements, and vice versa.
    fn epoch_cleanup(&mut self) {

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

        validator.on_stake_failed(amount); 

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

        validator.on_unstake_failed(amount);

        log_unstake_failed(&validator_id, amount);
    }
}
