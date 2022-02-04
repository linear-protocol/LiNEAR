use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    ext_contract, AccountId, Balance, EpochHeight, Promise,
    require, near_bindgen,
    json_types::{U128},
    collections::{UnorderedMap},
};
use crate::*;
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
    validators: UnorderedMap<AccountId, Validator>,
    total_weight: u16,
}

impl ValidatorPool {
    pub fn new() -> Self {
        Self {
            validators: UnorderedMap::new(b"vs".to_vec()),
            total_weight: 0,
        }
    }

    pub fn get_validator(
        &self,
        validator_id: &AccountId
    ) -> Option<Validator> {
        self.validators.get(validator_id)
    }

    pub fn add_validator(
        &mut self,
        validator_id: &AccountId,
        weight: u16
    ) {
        require!(
            self.get_validator(validator_id).is_none(),
            ERR_VALIDATOR_ALREADY_EXIST
        );

        let validator = Validator::new(
            validator_id.clone(),
            weight
        );

        self.validators.insert(
            validator_id,
            &validator
        );

        self.total_weight += weight;
    }

    pub fn remove_validator(
        &mut self,
        validator_id: &AccountId
    ) -> Validator {
        let validator = self.validators.remove(validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        self.total_weight -= validator.weight;

        return validator;
    }

    pub fn update_weight(
        &mut self,
        validator_id: &AccountId,
        weight: u16
    ) {
        let mut validator = self.validators.get(validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST);

        // update total weight
        self.total_weight = self.total_weight + weight - validator.weight;

        validator.weight = weight;
        self.validators.insert(
            validator_id,
            &validator
        );
    }

    pub fn get_validators(
        &self,
        offset: u16,
        limit: u16
    ) -> Vec<Validator> {
        self.validators.values()
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
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

#[near_bindgen]
impl LiquidStakingContract {
    pub fn add_validator(
        &mut self,
        validator_id: &AccountId,
        weight: u16
    ) {
        self.assert_owner();
        self.validator_pool.add_validator(
            validator_id,
            weight
        );
    }

    pub fn remove_validator(
        &mut self,
        validator_id: &AccountId
    ) -> Validator {
        self.assert_owner();
        self.validator_pool.remove_validator(validator_id)
    }

    pub fn update_weight(
        &mut self,
        validator_id: &AccountId,
        weight: u16
    ) {
        self.assert_owner();
        self.validator_pool.update_weight(
            validator_id,
            weight
        );
    }

    #[cfg(feature = "test")]
    pub fn get_total_weight(
        &self
    ) -> u16 {
        self.validator_pool.total_weight
    }

    pub fn get_validators(
        &self,
        offset: u16,
        limit: u16
    ) -> Vec<Validator> {
        self.assert_owner();
        self.validator_pool.get_validators(
            offset,
            limit
        )
    }
}

/// struct for staking pool validator
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Validator {
    pub account_id: AccountId,
    pub weight: u16,

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
        weight: u16,
    ) -> Self {
        Self {
            account_id,
            weight,
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
