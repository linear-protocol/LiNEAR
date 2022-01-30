use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    ext_contract, AccountId, Balance, EpochHeight, Promise,
    require,
    json_types::{U128},
    collections::{LookupMap},
};
use crate::types::*;
use crate::errors::*;
use crate::utils::*;

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
}

/// struct for staking pool validator
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Validator {
    pub account_id: AccountId,

    pub staked_amount: Balance,
    pub unstaked_amount: Balance,

    /// the epoch num when latest unstake action happened on this validator
    pub unstake_fired_epoch: EpochHeight,
    /// this is to save the last value of unstake_fired_epoch,
    /// so that when unstake revert we can restore it
    pub last_unstake_fired_epoch: EpochHeight,
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
            last_unstake_fired_epoch: 0,
        }
    }

    pub fn pending_release(& self) -> bool {
        let current_epoch = get_epoch_height();
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

    pub fn unstake(
        &mut self,
        amount: Balance
    ) -> Promise {
        require!(
            amount <= self.staked_amount,
            format!(
                "{}. staked: {}, requested: {}", 
                ERR_VALIDATOR_UNSTAKE_AMOUNT,
                self.staked_amount,
                amount
            )
        );

        // avoid unstake from a validator which is pending release
        require!(
            !self.pending_release(),
            ERR_VALIDATOR_UNSTAKE_WHEN_LOCKED
        );

        self.staked_amount -= amount;
        self.unstaked_amount += amount;
        self.last_unstake_fired_epoch = self.unstake_fired_epoch;
        self.unstake_fired_epoch = get_epoch_height();

        ext_staking_pool::unstake(
            amount.into(),
            self.account_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_UNSTAKE
        )
    }

    pub fn on_stake_failed(
        &mut self,
        amount: Balance
    ) {
        self.staked_amount -= amount;
    }

    pub fn on_unstake_failed(
        &mut self,
        amount: Balance
    ) {
        self.staked_amount += amount;
        self.unstaked_amount -= amount;
        self.unstake_fired_epoch = self.last_unstake_fired_epoch;
    }
}
