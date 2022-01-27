use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    ext_contract, AccountId, Balance, EpochHeight, env,
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
pub struct ValidatorPool {
    validators: LookupMap<AccountId, Validator>
}

impl ValidatorPool {
    pub fn get_candidate_to_stake(
        & self,
        amount: Balance,
    ) -> Validator {
        // TODO
        self.validators
            .get(&AccountId::new_unchecked("foo.near".to_string()))
            .unwrap()
    }

    pub fn get_candidate_to_unstake(
        & self,
        amount: Balance,
    ) -> Validator {
        // TODO
        self.validators
            .get(&AccountId::new_unchecked("bar.near".to_string()))
            .unwrap()
    }
}

/// struct for staking pool validator
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Validator {
    account_id: AccountId,

    staked_amount: Balance,
    unstaked_amount: Balance,

    /// the epoch num when latest unstake action happened on this validator
    unstake_fired_epoch: EpochHeight,
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
}
