use crate::*;
use near_sdk::{
    near_bindgen, Balance, log, is_promise_success,
};

use crate::errors::*;
use crate::types::*;
use crate::utils::*;
use crate::events::*;

const MIN_AMOUNT_TO_PERFORM_STAKE: Balance = 10 * ONE_NEAR;
/// min NEAR balance this contract should hold in order to cover
/// storage and contract call fees.
const CONTRACT_MIN_RESERVE_BALANCE: Balance = 30 * ONE_NEAR;

#[ext_contract(ext_self_action_cb)]
trait EpochActionCallbacks {
    fn validator_staked_callback(
        &mut self,
        validator_id: AccountId,
        amount: Balance
    );
}

/// Actions that should be called by off-chain actors
/// during each epoch.
#[near_bindgen]
impl LiquidStakingContract {
    pub fn epoch_stake(&mut self) {
        self.epoch_cleanup();

        if self.epoch_requested_stake_amount < MIN_AMOUNT_TO_PERFORM_STAKE {
            log!(format!("not enough NEAR to stake: {}", self.epoch_requested_stake_amount));
            return;
        }

        let (mut candidate, amount_to_stake) = self
            .validator_pool
            .get_candidate_to_stake(self.epoch_requested_stake_amount);

        if amount_to_stake > 0 {
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
    }

    pub fn epoch_unstake(&mut self) {
        self.epoch_cleanup();
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

/// callbacks
/// functions here SHOULD NOT PANIC!
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
}

pub fn a(&mut self) {
    a.count += 1;
    ext_contract::b();
}

pub fn a(&mut self) {
    ext_contract::b();
    a.count += 1;
}
