use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    ext_contract, AccountId, Balance, EpochHeight, env, Promise,
    json_types::{U128},
    collections::{LookupMap},
};
use crate::types::*;

#[ext_contract(ext_staking_pool)]
pub trait ExtStakingPool {
    fn get_account_staked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_total_balance(&self, account_id: AccountId) -> U128;

    fn deposit(&mut self);

    fn deposit_and_stake(&mut self);

    fn withdraw(&mut self, amount: U128);

    fn withdraw_all(&mut self);

    fn stake(&mut self, amount: U128);

    fn unstake(&mut self, amount: U128);

    fn unstake_all(&mut self);
}

/// A pool of validators.
/// The main function of this struct is to
/// store validator info and calculate the best candidate to stake/unstake.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidatorPool {
    validators: LookupMap<AccountId, Validator>
}

impl ValidatorPool {
    pub fn new() -> Self {
        Self {
            validators: LookupMap::new(b"vs".to_vec())
        }
    }

    pub fn get_validator(
        &self,
        validator_id: &AccountId
    ) -> Option<Validator> {
        self.validators.get(validator_id)
    }

    pub fn get_candidate_to_stake(
        & self,
        amount: Balance,
    ) -> (Validator, Balance) {
        // TODO
        let validator = self.validators
            .get(&AccountId::new_unchecked("foo.near".to_string()))
            .unwrap();

        (validator, amount)
    }

    pub fn get_candidate_to_unstake(
        & self,
        amount: Balance,
    ) -> (Validator, Balance) {
        // TODO
        let validator = self.validators
            .get(&AccountId::new_unchecked("bar.near".to_string()))
            .unwrap();

        (validator, amount)
    }

    pub fn get_num_epoch_to_unstake(amount: u128) -> EpochHeight {
        // the num of epoches can be doubled or trippled if not enough stake is available
        NUM_EPOCHS_TO_UNLOCK
    }
}

/// struct for staking pool validator
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Validator {
    pub account_id: AccountId,

    pub staked_amount: Balance,
    pub unstaked_amount: Balance,

    /// the epoch num when latest unstake action happened on this validator
    pub unstake_fired_epoch: EpochHeight,
}

impl Validator {
    pub fn new(
        account_id: AccountId,
    ) -> Self {
        Self {
            account_id,
            staked_amount: 0,
            unstaked_amount: 0,
            unstake_fired_epoch: 0,
        }
    }

    pub fn pending_release(& self) -> bool {
        let current_epoch = env::epoch_height();
        current_epoch >= self.unstake_fired_epoch &&
            current_epoch < self.unstake_fired_epoch + NUM_EPOCHS_TO_UNLOCK
    }

    pub fn deposit_and_stake(
        &mut self,
        amount: Balance
    ) -> Promise {
        self.staked_amount += amount;
        ext_staking_pool::deposit_and_stake(
            self.account_id.clone(),
            amount,
            GAS_EXT_DEPOSIT_AND_STAKE
        )
    }

    pub fn on_stake_failed(
        &mut self,
        amount: Balance
    ) {
        self.staked_amount -= amount;
    }
}
