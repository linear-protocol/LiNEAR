use crate::errors::*;
use crate::events::Event;
use crate::legacy::ValidatorV1_0_0;
use crate::legacy::ValidatorV1_3_0;
use crate::types::*;
use crate::utils::*;
use crate::*;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    ext_contract, is_promise_success,
    json_types::U128,
    log, near_bindgen, require, AccountId, Balance, EpochHeight, Promise,
};
use std::cmp::{max, min, Ordering};

const STAKE_SMALL_CHANGE_AMOUNT: Balance = ONE_NEAR;
const UNSTAKE_FACTOR: u128 = 2;
const MAX_SYNC_BALANCE_DIFF: Balance = 100;
const MAX_UPDATE_WEIGHTS_COUNT: usize = 300;

#[ext_contract(ext_staking_pool)]
pub trait ExtStakingPool {
    fn get_account_staked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_total_balance(&self, account_id: AccountId) -> U128;

    fn get_account(&self, account_id: AccountId) -> HumanReadableAccount;

    fn deposit(&mut self);

    fn deposit_and_stake(&mut self);

    fn withdraw(&mut self, amount: U128);

    fn withdraw_all(&mut self);

    fn stake(&mut self, amount: U128);

    fn unstake(&mut self, amount: U128);

    fn unstake_all(&mut self);
}

#[ext_contract(ext_whitelist)]
trait ExtWhitelist {
    fn is_whitelisted(&self, staking_pool_account_id: AccountId) -> bool;
}

#[ext_contract(ext_self_whitelist_cb)]
trait WhitelistCallback {
    fn is_whitelisted_callback(&mut self, validator_id: AccountId, weight: u16);
}

/// A pool of validators.
/// The main function of this struct is to
/// store validator info and calculate the best candidate to stake/unstake.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidatorPool {
    pub validators: UnorderedMap<AccountId, VersionedValidator>,
    pub total_weight: u16,
    pub total_base_stake_amount: Balance,
}

pub struct CandidateValidator {
    pub validator: Validator,
    pub amount: u128,
}

impl Default for ValidatorPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorPool {
    pub fn new() -> Self {
        Self {
            validators: UnorderedMap::new(StorageKey::ValidatorsV1),
            total_weight: 0,
            total_base_stake_amount: 0,
        }
    }

    pub fn count(&self) -> u64 {
        self.validators.len()
    }

    pub fn get_validator(&self, validator_id: &AccountId) -> Option<Validator> {
        self.validators.get(validator_id).map(|v| v.into())
    }

    pub fn save_validator(&mut self, validator: &Validator) {
        self.validators
            .insert(&validator.account_id, &(validator.clone().into()));
    }

    pub fn add_validator(&mut self, validator_id: &AccountId, weight: u16) -> Validator {
        require!(
            self.get_validator(validator_id).is_none(),
            ERR_VALIDATOR_ALREADY_EXIST
        );

        let validator = Validator::new(validator_id.clone(), weight);

        self.validators
            .insert(validator_id, &validator.clone().into());

        self.total_weight += weight;

        Event::ValidatorAdded {
            account_id: validator_id,
            weight,
        }
        .emit();

        validator
    }

    pub fn remove_validator(&mut self, validator_id: &AccountId) -> Validator {
        let validator: Validator = self
            .validators
            .remove(validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST)
            .into();

        // make sure this validator is not used at all
        require!(
            validator.staked_amount == 0 && validator.unstaked_amount == 0,
            ERR_VALIDATOR_IN_USE
        );

        self.total_weight -= validator.weight;
        self.total_base_stake_amount -= validator.base_stake_amount;

        Event::ValidatorRemoved {
            account_id: validator_id,
        }
        .emit();

        validator
    }

    pub fn update_weight(&mut self, validator_id: &AccountId, weight: u16) -> u16 {
        let mut validator: Validator = self
            .validators
            .get(validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST)
            .into();

        let old_weight = validator.weight;
        // update total weight
        self.total_weight = self.total_weight + weight - old_weight;

        validator.weight = weight;
        self.validators.insert(validator_id, &validator.into());

        old_weight
    }

    /// Update base stake amount of the validator
    pub fn update_base_stake_amount(&mut self, validator_id: &AccountId, amount: Balance) {
        let mut validator: Validator = self
            .validators
            .get(validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST)
            .into();

        let old_base_stake_amount = validator.base_stake_amount;
        // update total base stake amount
        self.total_base_stake_amount =
            self.total_base_stake_amount + amount - old_base_stake_amount;

        validator.base_stake_amount = amount;
        self.validators.insert(validator_id, &validator.into());

        Event::ValidatorUpdatedBaseStakeAmount {
            account_id: validator_id,
            old_base_stake_amount: &old_base_stake_amount.into(),
            new_base_stake_amount: &amount.into(),
        }
        .emit();
    }

    pub fn get_validators(&self, offset: u64, limit: u64) -> Vec<Validator> {
        let keys = self.validators.keys_as_vector();
        (offset..std::cmp::min(offset + limit, keys.len()))
            .map(|index| self.get_validator(&keys.get(index).unwrap()).unwrap())
            .collect()
    }

    pub fn get_candidate_to_stake(
        &self,
        amount: Balance,
        total_staked_near_amount: Balance,
    ) -> Option<CandidateValidator> {
        let mut candidate = None;
        let mut amount_to_stake: Balance = 0;

        for (_, validator) in self.validators.iter() {
            let validator = validator.into();
            let target_amount =
                self.validator_target_stake_amount(total_staked_near_amount, &validator);
            if validator.staked_amount < target_amount {
                let delta = min(target_amount - validator.staked_amount, amount);
                if delta > amount_to_stake {
                    amount_to_stake = delta;
                    candidate = Some(validator);
                }
            }
        }

        if amount_to_stake > 0 && amount - amount_to_stake < STAKE_SMALL_CHANGE_AMOUNT {
            amount_to_stake = amount;
        }

        candidate.map(|candidate| CandidateValidator {
            validator: candidate,
            amount: amount_to_stake,
        })
    }

    pub fn get_candidate_to_unstake(
        &self,
        amount: Balance,
        total_staked_near_amount: Balance,
    ) -> Option<CandidateValidator> {
        let mut candidate = None;
        let mut amount_to_unstake: Balance = 0;

        for (_, validator) in self.validators.iter() {
            let validator: Validator = validator.into();
            if validator.pending_release() {
                continue;
            }

            let validator_id = validator.clone().account_id;
            let staked_amount = validator.staked_amount;
            let target_amount =
                self.validator_target_stake_amount(total_staked_near_amount, &validator);
            if validator.staked_amount > target_amount {
                let delta = min3(
                    // more NEAR than delta will be unstaked to
                    // prevent the need to unstake from all validators,
                    // which blocks all of them.
                    UNSTAKE_FACTOR * (validator.staked_amount - target_amount),
                    amount,
                    validator.staked_amount,
                );
                if delta > amount_to_unstake {
                    amount_to_unstake = delta;
                    candidate = Some(validator);
                }
            }

            log!(
                "check unstake candidate {} with staked amount {} and target {}",
                validator_id,
                staked_amount,
                target_amount
            );
        }

        // if the amount left is too small, we try to unstake them at once
        if amount_to_unstake > 0 && amount - amount_to_unstake < STAKE_SMALL_CHANGE_AMOUNT {
            amount_to_unstake = min(amount, candidate.as_ref().unwrap().staked_amount);
        }

        candidate.map(|candidate| CandidateValidator {
            validator: candidate,
            amount: amount_to_unstake,
        })
    }

    /// **formula: target stake amount = base stake amount + dynamic stake amount.**
    ///
    /// In this model, we ensure the sum of target stake amount is equal to the total staked amount,
    /// and prioritize base stake amount over dynamic stake amount.
    ///
    /// If total staked NEAR amount >= total base stake amount,
    /// 1. satisfy the base stake amount set by validator manager;
    /// 2. calculate dynamic stake amount proportional to weight
    ///
    /// If total staked NEAR amount < total base stake amount,
    /// 1. set dynamic stake amount to 0;
    /// 2. calculate the base stake amount proportionally
    pub fn validator_target_stake_amount(
        &self,
        total_staked_near_amount: Balance,
        validator: &Validator,
    ) -> Balance {
        let base_stake_amount = if total_staked_near_amount >= self.total_base_stake_amount {
            validator.base_stake_amount
        } else {
            (U256::from(validator.base_stake_amount) * U256::from(total_staked_near_amount)
                / U256::from(self.total_base_stake_amount))
            .as_u128()
        };
        // If not enough staked NEAR, satisfy the base stake amount first (set dynamic stake amount to 0)
        let dynamic_stake_amount =
            if validator.weight == 0 || total_staked_near_amount <= self.total_base_stake_amount {
                0
            } else {
                (U256::from(total_staked_near_amount - self.total_base_stake_amount)
                    * U256::from(validator.weight)
                    / U256::from(self.total_weight))
                .as_u128()
            };
        base_stake_amount + dynamic_stake_amount
    }

    pub fn get_num_epoch_to_unstake(&self, amount: u128) -> EpochHeight {
        let mut available_amount: Balance = 0;
        let mut total_staked_amount: Balance = 0;
        for validator in self.validators.values() {
            let validator: Validator = validator.into();
            total_staked_amount += validator.staked_amount;

            if !validator.pending_release() && validator.staked_amount > 0 {
                available_amount += validator.staked_amount;
            }

            // found enough balance to unstake from available validators
            if available_amount >= amount {
                return NUM_EPOCHS_TO_UNLOCK;
            }
        }

        // nothing is actually staked, all balance should be available now
        // still leave a buffer for the user
        if total_staked_amount == 0 {
            return NUM_EPOCHS_TO_UNLOCK;
        }

        // no enough available validators to unstake
        // double the unstake wating time
        2 * NUM_EPOCHS_TO_UNLOCK
    }
}

// V2 Unstake strategy impl
impl ValidatorPool {
    /// Strategy
    /// Step 1. If there is a validator whose `delta` is greater than `total_amount_to_unstake`,
    ///         it will be selected (If more than one validator match the condition, select one
    ///         which has smallest `delta`).
    /// Step 2. If we didn't find any valid validator in step 1, we should select a validator which
    ///         has largest `delta / target_amount` (If `target_amount` is 0, select one which
    ///         has largest `delta`).
    pub fn get_candidate_to_unstake_v2(
        &self,
        total_amount_to_unstake: Balance,
        total_staked_near_amount: Balance,
    ) -> Option<CandidateValidator> {
        let mut candidate_validators = self.extract_candidate_validators(total_staked_near_amount);
        if candidate_validators.is_empty() {
            return None;
        }

        Self::sort_candidate_validators_by_delta_asc(&mut candidate_validators);

        let candidate = candidate_validators
            .iter()
            .find(|(_validator, _target_amount, delta)| *delta >= total_amount_to_unstake);

        if let Some((validator, _target_amount, _delta)) = candidate {
            return Some(CandidateValidator {
                validator: validator.clone(),
                amount: total_amount_to_unstake,
            });
        };

        Self::sort_candidate_validators_by_ratio_of_delta_to_target_desc(&mut candidate_validators);

        candidate_validators
            .first()
            .map(|(validator, target_amount, delta)| {
                let amount_to_unstake = min3(
                    // unstake no more than total requirement
                    total_amount_to_unstake,
                    // When `stake < 1.5 * target`, unstake `target_amount / 2`, else unstake `delta`
                    max(target_amount / 2, *delta),
                    // guaranteed minimum staked amount even if `total_staked_near_amount` is less than `total_base_stake_amount`
                    validator.staked_amount.saturating_sub(min(
                        (U256::from(validator.base_stake_amount)
                            * U256::from(total_staked_near_amount))
                        .checked_div(U256::from(self.total_base_stake_amount))
                        .unwrap_or_default()
                        .as_u128(),
                        validator.base_stake_amount,
                    )),
                );
                CandidateValidator {
                    validator: validator.clone(),
                    amount: amount_to_unstake,
                }
            })
    }

    // extract from validator list, return Vec<(validator, target_amount, delta)>
    fn extract_candidate_validators(
        &self,
        total_staked_near_amount: Balance,
    ) -> Vec<(Validator, Balance, Balance)> {
        self.validators
            .values()
            .map(|versioned_validator| {
                let validator = Validator::from(versioned_validator);
                let target_amount =
                    self.validator_target_stake_amount(total_staked_near_amount, &validator);
                (validator, target_amount)
            })
            .filter(|(validator, target_amount)| {
                // validator is not in pending release
                !validator.pending_release()
                    // delta must > 0
                    && validator.staked_amount > *target_amount
            })
            .map(|(validator, target_amount)| {
                let delta = validator.staked_amount - target_amount; // safe sub
                (validator, target_amount, delta)
            })
            .collect()
    }

    // sort candidate validator by delta in ascending
    fn sort_candidate_validators_by_delta_asc(
        candidate_validators: &mut [(Validator, Balance, Balance)],
    ) {
        candidate_validators.sort_by(
            |(_validator_1, _target_amount_1, delta_1),
             (_validator_2, _target_amount_2, delta_2)| { delta_1.cmp(delta_2) },
        );
    }

    // sort candidate validator by delta in descending
    fn sort_candidate_validators_by_ratio_of_delta_to_target_desc(
        candidate_validators: &mut [(Validator, Balance, Balance)],
    ) {
        candidate_validators.sort_by(
            |(_validator_1, target_amount_1, delta_1), (_validator_2, target_amount_2, delta_2)| {
                let target_amount_1 = *target_amount_1;
                let target_amount_2 = *target_amount_2;

                if target_amount_1 == 0 && target_amount_2 == 0 {
                    delta_2.cmp(delta_1)
                } else if target_amount_1 != 0 && target_amount_2 == 0 {
                    Ordering::Greater
                } else if target_amount_1 == 0 && target_amount_2 != 0 {
                    Ordering::Less
                } else {
                    // We can simplify `(delta_2 / target_amount_2) cmp (delta_1 / target_amount_1)`
                    // to `(delta_2 * target_amount_1) cmp (delta_1 * target_amount_2)`
                    let mul_1 = U256::from(*delta_1) * U256::from(target_amount_2);
                    let mul_2 = U256::from(*delta_2) * U256::from(target_amount_1);
                    mul_2.cmp(&mul_1)
                }
            },
        );
    }
}

fn min3(x: u128, y: u128, z: u128) -> u128 {
    min(x, min(y, z))
}

#[near_bindgen]
impl LiquidStakingContract {
    // --- Call Functions ---

    pub fn add_validator(&mut self, validator_id: AccountId, weight: u16) {
        self.assert_running();
        self.assert_manager();
        self.add_whitelisted_validator(&validator_id, weight);
    }

    pub fn add_validators(&mut self, validator_ids: Vec<AccountId>, weights: Vec<u16>) {
        self.assert_running();
        self.assert_manager();
        require!(validator_ids.len() == weights.len(), ERR_BAD_VALIDATOR_LIST);
        for i in 0..validator_ids.len() {
            self.add_whitelisted_validator(&validator_ids[i], weights[i]);
        }
    }

    /// Add a new validator only if it's whitelisted
    fn add_whitelisted_validator(&mut self, validator_id: &AccountId, weight: u16) {
        let whitelist_id = self
            .whitelist_account_id
            .as_ref()
            .expect(ERR_VALIDATOR_WHITELIST_NOT_SET);

        ext_whitelist::is_whitelisted(
            validator_id.clone(),
            whitelist_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_WHITELIST,
        )
        .then(ext_self_whitelist_cb::is_whitelisted_callback(
            validator_id.clone(),
            weight,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_CB_WHITELIST,
        ));
    }

    #[private]
    pub fn is_whitelisted_callback(
        &mut self,
        validator_id: AccountId,
        weight: u16,
        #[callback] whitelisted: bool,
    ) {
        require!(
            whitelisted,
            format!(
                "{}. {}",
                ERR_VALIDATOR_NOT_WHITELISTED,
                validator_id.clone()
            )
        );

        self.validator_pool.add_validator(&validator_id, weight);
    }

    pub fn remove_validator(&mut self, validator_id: AccountId) -> Validator {
        self.assert_running();
        self.assert_manager();
        self.validator_pool.remove_validator(&validator_id)
    }

    pub fn update_weight(&mut self, validator_id: AccountId, weight: u16) {
        self.assert_running();
        self.assert_manager();
        let old_weight = self.validator_pool.update_weight(&validator_id, weight);
        Event::ValidatorsUpdatedWeights {
            account_ids: vec![&validator_id],
            old_weights: vec![old_weight],
            new_weights: vec![weight],
        }
        .emit();
    }

    pub fn update_weights(&mut self, validator_ids: Vec<AccountId>, weights: Vec<u16>) {
        self.assert_running();
        self.assert_manager();
        require!(validator_ids.len() == weights.len(), ERR_BAD_VALIDATOR_LIST);

        require!(
            validator_ids.len() <= MAX_UPDATE_WEIGHTS_COUNT,
            format!(
                "The number of validators to be updated at a time cannot exceed {}",
                MAX_UPDATE_WEIGHTS_COUNT
            )
        );

        let mut account_ids = Vec::new();
        let mut old_weights = Vec::new();
        let mut new_weights = Vec::new();

        for i in 0..validator_ids.len() {
            let old_weight = self
                .validator_pool
                .update_weight(&validator_ids[i], weights[i]);
            account_ids.push(&validator_ids[i]);
            old_weights.push(old_weight);
            new_weights.push(weights[i]);
        }

        Event::ValidatorsUpdatedWeights {
            account_ids,
            old_weights,
            new_weights,
        }
        .emit();
    }

    pub fn update_base_stake_amounts(&mut self, validator_ids: Vec<AccountId>, amounts: Vec<U128>) {
        self.assert_running();
        self.assert_manager();
        require!(validator_ids.len() == amounts.len(), ERR_BAD_VALIDATOR_LIST);
        for i in 0..validator_ids.len() {
            self.validator_pool
                .update_base_stake_amount(&validator_ids[i], amounts[i].into());
        }
    }

    // --- View Functions ---

    #[cfg(feature = "test")]
    pub fn get_total_weight(&self) -> u16 {
        self.validator_pool.total_weight
    }

    pub fn get_validator(&self, validator_id: AccountId) -> ValidatorInfo {
        self.validator_pool
            .get_validator(&validator_id)
            .expect(ERR_VALIDATOR_NOT_EXIST)
            .get_info(&self.validator_pool, self.total_staked_near_amount)
    }

    pub fn get_validators(&self, offset: u64, limit: u64) -> Vec<ValidatorInfo> {
        self.validator_pool
            .get_validators(offset, limit)
            .iter()
            .map(|v| v.get_info(&self.validator_pool, self.total_staked_near_amount))
            .collect()
    }
}

// Drain Validator

#[ext_contract(ext_self_validator_drain_cb)]
trait ValidatorDrainCallbacks {
    fn validator_drain_unstaked_callback(&mut self, validator_id: AccountId, amount: U128);

    fn validator_drain_withdraw_callback(&mut self, validator_id: AccountId, amount: U128);
}

#[near_bindgen]
impl LiquidStakingContract {
    /// This method is designed to drain a validator.
    /// The weight of target validator should be set to 0 before calling this.
    /// And a following call to drain_withdraw MUST be made after 4 epoches.
    pub fn drain_unstake(&mut self, validator_id: AccountId) {
        self.assert_running();
        self.assert_manager();

        // make sure enough gas was given
        let min_gas = GAS_DRAIN_UNSTAKE + GAS_EXT_UNSTAKE + GAS_CB_VALIDATOR_UNSTAKED;
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
        // 2. has base stake amount set to 0
        // 3. not in pending release
        // 4. has not unstaked balance (because this part is from user's unstake request)
        // 5. not in draining process
        require!(validator.weight == 0, ERR_NON_ZERO_WEIGHT);
        require!(
            validator.base_stake_amount == 0,
            ERR_NON_ZERO_BASE_STAKE_AMOUNT
        );
        require!(
            !validator.pending_release(),
            ERR_VALIDATOR_UNSTAKE_WHEN_LOCKED
        );
        // in practice we allow 1 NEAR due to the precision of stake operation
        require!(
            validator.unstaked_amount < ONE_NEAR,
            ERR_BAD_UNSTAKED_AMOUNT
        );
        require!(!validator.draining, ERR_DRAINING);

        let unstake_amount = validator.staked_amount;

        Event::DrainUnstakeAttempt {
            validator_id: &validator_id,
            amount: &U128(unstake_amount),
        }
        .emit();

        // perform actual unstake
        validator
            .unstake(&mut self.validator_pool, unstake_amount)
            .then(
                ext_self_validator_drain_cb::validator_drain_unstaked_callback(
                    validator.account_id,
                    unstake_amount.into(),
                    env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_CB_VALIDATOR_UNSTAKED,
                ),
            );
    }

    /// Withdraw from a drained validator
    pub fn drain_withdraw(&mut self, validator_id: AccountId) {
        self.assert_running();

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
        // 2. has base stake amount set to 0
        // 3. has no staked balance
        // 4. not pending release
        // 5. in draining process
        require!(validator.weight == 0, ERR_NON_ZERO_WEIGHT);
        require!(
            validator.base_stake_amount == 0,
            ERR_NON_ZERO_BASE_STAKE_AMOUNT
        );
        require!(validator.staked_amount == 0, ERR_NON_ZERO_STAKED_AMOUNT);
        require!(
            !validator.pending_release(),
            ERR_VALIDATOR_WITHDRAW_WHEN_LOCKED
        );
        require!(validator.draining, ERR_NOT_IN_DRAINING);

        let amount = validator.unstaked_amount;

        Event::DrainWithdrawAttempt {
            validator_id: &validator_id,
            amount: &U128(amount),
        }
        .emit();

        validator.withdraw(&mut self.validator_pool, amount).then(
            ext_self_validator_drain_cb::validator_drain_withdraw_callback(
                validator.account_id.clone(),
                amount.into(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_CB_VALIDATOR_WITHDRAW,
            ),
        );
    }

    #[private]
    pub fn validator_drain_unstaked_callback(&mut self, validator_id: AccountId, amount: U128) {
        let amount = amount.into();
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .unwrap_or_else(|| panic!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        if is_promise_success() {
            validator.on_unstake_success(&mut self.validator_pool, amount);
            validator.set_draining(&mut self.validator_pool, true);

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
    pub fn validator_drain_withdraw_callback(&mut self, validator_id: AccountId, amount: U128) {
        let amount = amount.into();
        let mut validator = self
            .validator_pool
            .get_validator(&validator_id)
            .unwrap_or_else(|| panic!("{}: {}", ERR_VALIDATOR_NOT_EXIST, &validator_id));

        if is_promise_success() {
            validator.set_draining(&mut self.validator_pool, false);

            Event::DrainWithdrawSuccess {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();

            // those funds need to be restaked, so we add them back to epoch request
            self.epoch_requested_stake_amount += amount;
        } else {
            // withdraw failed, revert
            validator.on_withdraw_failed(&mut self.validator_pool, amount);

            Event::DrainWithdrawFailed {
                validator_id: &validator_id,
                amount: &U128(amount),
            }
            .emit();
        }
    }
}

/// How to add a new variant for VersionedValidator:
/// 1. Put the current definition of Validator into legacy.rs as `ValidatorVx_x_x`
/// 2. Update the current Validator struct
/// 3. Implement `From<ValidatorVx_x_x> for Validator` so that we can migrate
///    from the previous version of Validator to the latest.
/// 4. Insert a new variant of VersionedValidator just BEFORE `Current(Validator)`.
///    It stands for the previous version of Validator, which should be version
///    that production is using at the time of writing the code.
///    Due to the fact that Borsh use variant index instead of name to serialize,
///    the new variant will be deserialzed to the previous version correctly after
///    new code is deployed.
/// 5. Update `impl From<VersionedValidator> for Validator`, to match the new variant.
#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedValidator {
    V0(ValidatorV1_0_0),
    V1(ValidatorV1_3_0),
    Current(Validator),
}

impl From<Validator> for VersionedValidator {
    fn from(v: Validator) -> Self {
        VersionedValidator::Current(v)
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

    /// The base stake amount on this validator.
    pub base_stake_amount: Balance,

    /// the epoch num when latest unstake action happened on this validator
    pub unstake_fired_epoch: EpochHeight,
    /// this is to save the last value of unstake_fired_epoch,
    /// so that when unstake revert we can restore it
    pub last_unstake_fired_epoch: EpochHeight,

    /// Whether the validator is in draining process
    pub draining: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorInfo {
    pub account_id: AccountId,
    pub weight: u16,
    pub base_stake_amount: U128,
    pub target_stake_amount: U128,
    pub staked_amount: U128,
    pub unstaked_amount: U128,
    pub pending_release: bool,
    pub draining: bool,
}

impl From<VersionedValidator> for Validator {
    fn from(value: VersionedValidator) -> Self {
        match value {
            VersionedValidator::Current(v) => v,
            VersionedValidator::V1(v1) => v1.into(),
            VersionedValidator::V0(v0) => v0.into(),
        }
    }
}

impl Validator {
    pub fn new(account_id: AccountId, weight: u16) -> Self {
        Self {
            account_id,
            weight,
            base_stake_amount: 0,
            staked_amount: 0,
            unstaked_amount: 0,
            unstake_fired_epoch: 0,
            last_unstake_fired_epoch: 0,
            draining: false,
        }
    }

    pub fn get_info(
        &self,
        pool: &ValidatorPool,
        total_staked_near_amount: Balance,
    ) -> ValidatorInfo {
        ValidatorInfo {
            account_id: self.account_id.clone(),
            weight: self.weight,
            base_stake_amount: self.base_stake_amount.into(),
            target_stake_amount: pool
                .validator_target_stake_amount(total_staked_near_amount, self)
                .into(),
            staked_amount: self.staked_amount.into(),
            unstaked_amount: self.unstaked_amount.into(),
            pending_release: self.pending_release(),
            draining: self.draining,
        }
    }

    pub fn total_balance(&self) -> Balance {
        self.staked_amount + self.unstaked_amount
    }

    /// whether the validator is in unstake releasing period.
    pub fn pending_release(&self) -> bool {
        let current_epoch = get_epoch_height();
        current_epoch >= self.unstake_fired_epoch
            && current_epoch < self.unstake_fired_epoch + NUM_EPOCHS_TO_UNLOCK
    }

    pub fn deposit_and_stake(&mut self, amount: Balance) -> Promise {
        ext_staking_pool::deposit_and_stake(
            self.account_id.clone(),
            amount,
            GAS_EXT_DEPOSIT_AND_STAKE,
        )
    }

    pub fn on_stake_success(&mut self, pool: &mut ValidatorPool, amount: Balance) {
        self.staked_amount += amount;
        pool.save_validator(self);
    }

    pub fn unstake(&mut self, pool: &mut ValidatorPool, amount: Balance) -> Promise {
        require!(
            amount <= self.staked_amount,
            format!(
                "{}. staked: {}, requested: {}",
                ERR_VALIDATOR_UNSTAKE_AMOUNT, self.staked_amount, amount
            )
        );

        // avoid unstake from a validator which is pending release
        require!(!self.pending_release(), ERR_VALIDATOR_UNSTAKE_WHEN_LOCKED);

        self.staked_amount -= amount;
        self.last_unstake_fired_epoch = self.unstake_fired_epoch;
        self.unstake_fired_epoch = get_epoch_height();

        pool.save_validator(self);

        ext_staking_pool::unstake(
            amount.into(),
            self.account_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_UNSTAKE,
        )
    }

    pub fn on_unstake_success(&mut self, pool: &mut ValidatorPool, amount: Balance) {
        self.unstaked_amount += amount;
        pool.save_validator(self);
    }

    pub fn on_unstake_failed(&mut self, pool: &mut ValidatorPool, amount: Balance) {
        self.staked_amount += amount;
        self.unstake_fired_epoch = self.last_unstake_fired_epoch;
        pool.save_validator(self);
    }

    pub fn refresh_total_balance(&self) -> Promise {
        ext_staking_pool::get_account_total_balance(
            env::current_account_id(),
            self.account_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_GET_BALANCE,
        )
    }

    pub fn on_new_total_balance(&mut self, pool: &mut ValidatorPool, new_total_balance: Balance) {
        // sync base stake amount
        self.sync_base_stake_amount(pool, new_total_balance);
        // update staked amount
        self.staked_amount = new_total_balance - self.unstaked_amount;
        pool.save_validator(self);
    }

    pub fn sync_account_balance(&self) -> Promise {
        ext_staking_pool::get_account(
            env::current_account_id(),
            self.account_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_GET_ACCOUNT,
        )
    }

    pub fn on_sync_account_balance(
        &mut self,
        pool: &mut ValidatorPool,
        staked_balance: Balance,
        unstaked_balance: Balance,
    ) {
        // allow at most 1 yN diff in total balance
        let new_total_balance = staked_balance + unstaked_balance;
        require!(
            abs_diff_eq(new_total_balance, self.total_balance(), 1),
            format!(
                "{}. new: {}, old: {}",
                ERR_SYNC_BALANCE_BAD_TOTAL,
                new_total_balance,
                self.total_balance()
            )
        );

        // allow at most 100 yN diff in staked/unstaked balance
        require!(
            abs_diff_eq(staked_balance, self.staked_amount, MAX_SYNC_BALANCE_DIFF),
            format!(
                "{}. new: {}, old: {}",
                ERR_SYNC_BALANCE_BAD_STAKED, staked_balance, self.staked_amount
            )
        );
        require!(
            abs_diff_eq(
                unstaked_balance,
                self.unstaked_amount,
                MAX_SYNC_BALANCE_DIFF
            ),
            format!(
                "{}. new: {}, old: {}",
                ERR_SYNC_BALANCE_BAD_UNSTAKED, unstaked_balance, self.unstaked_amount
            )
        );

        // sync base stake amount
        self.sync_base_stake_amount(pool, new_total_balance);

        // update balance
        self.staked_amount = staked_balance;
        self.unstaked_amount = unstaked_balance;

        pool.save_validator(self);
    }

    fn sync_base_stake_amount(&mut self, pool: &mut ValidatorPool, new_total_balance: Balance) {
        let old_total_balance = self.staked_amount + self.unstaked_amount;
        // If no balance, or no base stake amount set, no need to update base stake amount
        if old_total_balance != 0 && self.base_stake_amount != 0 {
            let old_base_stake_amount = self.base_stake_amount;
            self.base_stake_amount = (U256::from(old_base_stake_amount)
                * U256::from(new_total_balance)
                / U256::from(old_total_balance))
            .as_u128();
            pool.total_base_stake_amount =
                pool.total_base_stake_amount + self.base_stake_amount - old_base_stake_amount;
        }
    }

    pub fn withdraw(&mut self, pool: &mut ValidatorPool, amount: Balance) -> Promise {
        require!(
            self.unstaked_amount >= amount,
            ERR_NO_ENOUGH_WITHDRAW_BALANCE
        );
        require!(!self.pending_release(), ERR_VALIDATOR_WITHDRAW_WHEN_LOCKED);

        self.unstaked_amount -= amount;
        pool.save_validator(self);

        ext_staking_pool::withdraw(
            amount.into(),
            self.account_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_WITHDRAW,
        )
    }

    pub fn on_withdraw_failed(&mut self, pool: &mut ValidatorPool, amount: Balance) {
        self.unstaked_amount += amount;
        pool.save_validator(self);
    }

    pub fn set_draining(&mut self, pool: &mut ValidatorPool, draining: bool) {
        self.draining = draining;
        pool.save_validator(self);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_candidate_select() {
        let mut validator_pool = ValidatorPool::new();

        let mut foo = validator_pool.add_validator(&AccountId::new_unchecked("foo".to_string()), 1);
        let mut bar = validator_pool.add_validator(&AccountId::new_unchecked("bar".to_string()), 1);
        let mut zoo = validator_pool.add_validator(&AccountId::new_unchecked("zoo".to_string()), 2);

        // manually set staked amounts
        foo.staked_amount = 100 * ONE_NEAR; // target is 150
        bar.staked_amount = 200 * ONE_NEAR; // target is 150
        zoo.staked_amount = 200 * ONE_NEAR; // target is 300
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 600 in total, 500 already staked, 100 to stake,
        // each weight point should be 150, thus zoo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_stake(100 * ONE_NEAR, 600 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, zoo.account_id);
        assert_eq!(candidate.amount, 100 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 0; // target is 150
        bar.staked_amount = 200 * ONE_NEAR; // target is 150
        zoo.staked_amount = 300 * ONE_NEAR; // target is 150
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 600 in total, 500 already staked, 100 to stake,
        // each weight point should be 150, thus zoo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_stake(100 * ONE_NEAR, 600 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, foo.account_id);
        assert_eq!(candidate.amount, 100 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 200 * ONE_NEAR; // target is 150
        bar.staked_amount = 200 * ONE_NEAR; // target is 150
        zoo.staked_amount = 300 * ONE_NEAR; // target is 300
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // in case no staking is needed

        assert!(validator_pool
            .get_candidate_to_stake(100 * ONE_NEAR, 600 * ONE_NEAR)
            .is_none());
    }

    #[test]
    fn test_stake_candidate_select_with_base_stake_amount() {
        let mut validator_pool = ValidatorPool::new();

        let mut foo = validator_pool.add_validator(&AccountId::new_unchecked("foo".to_string()), 1);
        let mut bar = validator_pool.add_validator(&AccountId::new_unchecked("bar".to_string()), 1);
        let mut zoo = validator_pool.add_validator(&AccountId::new_unchecked("zoo".to_string()), 2);

        // set foo's base stake amount to 200
        validator_pool.update_base_stake_amount(&foo.account_id, 200 * ONE_NEAR);
        foo = validator_pool
            .get_validator(&AccountId::new_unchecked("foo".to_string()))
            .unwrap();

        // 1. total stake amount >= total base stake amount

        // manually set staked amounts
        foo.staked_amount = 150 * ONE_NEAR; // target is 400
        bar.staked_amount = 200 * ONE_NEAR; // target is 200
        zoo.staked_amount = 200 * ONE_NEAR; // target is 400
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 1000 in total, 550 already staked, 250 to stake,
        // each weight point should be 200, thus foo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_stake(250 * ONE_NEAR, 1000 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, foo.account_id);
        assert_eq!(candidate.amount, 250 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 0; // target is 350
        bar.staked_amount = 200 * ONE_NEAR; // target is 150
        zoo.staked_amount = 300 * ONE_NEAR; // target is 300
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 800 in total, 500 already staked, 200 to stake,
        // each weight point should be 150, thus foo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_stake(200 * ONE_NEAR, 800 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, foo.account_id);
        assert_eq!(candidate.amount, 200 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 500 * ONE_NEAR; // target is 400
        bar.staked_amount = 300 * ONE_NEAR; // target is 200
        zoo.staked_amount = 500 * ONE_NEAR; // target is 400
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // in case no staking is needed

        assert!(validator_pool
            .get_candidate_to_stake(100 * ONE_NEAR, 1000 * ONE_NEAR)
            .is_none());

        // 2. total stake amount < total base stake amount

        // reset staked amount
        foo.staked_amount = 0; // target is 100
        bar.staked_amount = 20 * ONE_NEAR; // target is 0
        zoo.staked_amount = 30 * ONE_NEAR; // target is 0
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 100 in total, 50 already staked, 50 to stake,
        // the total stake amount is less than total base stake amount, satisfay base stake amount first.
        // thus foo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_stake(50 * ONE_NEAR, 100 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, foo.account_id);
        assert_eq!(candidate.amount, 50 * ONE_NEAR);

        // set bar's base stake amount to 100
        validator_pool.update_base_stake_amount(&bar.account_id, 100 * ONE_NEAR);
        bar = validator_pool
            .get_validator(&AccountId::new_unchecked("bar".to_string()))
            .unwrap();

        // reset staked amount
        foo.staked_amount = 75 * ONE_NEAR; // target is 100
        bar.staked_amount = 20 * ONE_NEAR; // target is 50
        zoo.staked_amount = 5 * ONE_NEAR; // target is 0
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 150 in total, 100 already staked, 50 to stake,
        // the total stake amount is less than total base stake amount, satisfay base stake amount first.
        // thus bar is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_stake(50 * ONE_NEAR, 150 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, bar.account_id);
        assert_eq!(candidate.amount, 30 * ONE_NEAR);
    }

    #[test]
    fn test_unstake_candidate_select() {
        let mut validator_pool = ValidatorPool::new();

        let mut foo = validator_pool.add_validator(&AccountId::new_unchecked("foo".to_string()), 1);
        let mut bar = validator_pool.add_validator(&AccountId::new_unchecked("bar".to_string()), 1);
        let mut zoo = validator_pool.add_validator(&AccountId::new_unchecked("zoo".to_string()), 2);

        // manually set staked amounts
        foo.staked_amount = 108 * ONE_NEAR; // target is 100
        bar.staked_amount = 107 * ONE_NEAR; // target is 100
        zoo.staked_amount = 210 * ONE_NEAR; // target is 200
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 425 already staked, 25 to unstake, target total 400,
        // each weight point should be 100, thus zoo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_unstake(25 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, zoo.account_id);
        assert_eq!(candidate.amount, 20 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 100; // target is 100
        bar.staked_amount = 200 * ONE_NEAR; // target is 100
        zoo.staked_amount = 200 * ONE_NEAR; // target is 200
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 500 already staked, 100 to unstake, target total 400,
        // each weight point should be 100, thus bar is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_unstake(100 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, bar.account_id);
        assert_eq!(candidate.amount, 100 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 100;
        bar.staked_amount = 100;
        zoo.staked_amount = 100;
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // in case no unstaking is needed

        assert!(validator_pool.get_candidate_to_unstake(100, 400).is_none());
    }

    #[test]
    fn test_unstake_candidate_select_with_base_stake_amount() {
        let mut validator_pool = ValidatorPool::new();

        let mut foo = validator_pool.add_validator(&AccountId::new_unchecked("foo".to_string()), 1);
        let mut bar = validator_pool.add_validator(&AccountId::new_unchecked("bar".to_string()), 1);
        let mut zoo = validator_pool.add_validator(&AccountId::new_unchecked("zoo".to_string()), 2);

        // set foo's base stake amount to 200
        validator_pool.update_base_stake_amount(&foo.account_id, 200 * ONE_NEAR);
        foo = validator_pool
            .get_validator(&AccountId::new_unchecked("foo".to_string()))
            .unwrap();

        // 1. total stake amount >= total base stake amount

        // manually set staked amounts
        foo.staked_amount = 100 * ONE_NEAR; // target is 250
        bar.staked_amount = 100 * ONE_NEAR; // target is 50
        zoo.staked_amount = 210 * ONE_NEAR; // target is 100
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 510 already staked, 110 to unstake, target total 400,
        // each weight point should be 50, thus zoo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_unstake(110 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, zoo.account_id);
        assert_eq!(candidate.amount, 110 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 100 * ONE_NEAR; // target is 250
        bar.staked_amount = 200 * ONE_NEAR; // target is 50
        zoo.staked_amount = 200 * ONE_NEAR; // target is 100
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 500 already staked, 100 to unstake, target total 400,
        // each weight point should be 50, thus bar is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_unstake(100 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, bar.account_id);
        assert_eq!(candidate.amount, 100 * ONE_NEAR);

        // reset staked amount
        foo.staked_amount = 100 * ONE_NEAR;
        bar.staked_amount = 50 * ONE_NEAR;
        zoo.staked_amount = 100 * ONE_NEAR;
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // in case no unstaking is needed

        assert!(validator_pool
            .get_candidate_to_unstake(100 * ONE_NEAR, 400 * ONE_NEAR)
            .is_none());

        // 2. total stake amount < total base stake amount

        // reset staked amount
        foo.staked_amount = 100 * ONE_NEAR; // target is 200
        bar.staked_amount = 150 * ONE_NEAR; // target is 0
        zoo.staked_amount = 200 * ONE_NEAR; // target is 0
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 450 already staked, 250 to unstake, target total 200,
        // the total stake amount is less than total base stake amount, satisfay base stake amount first,
        // thus zoo is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_unstake(200 * ONE_NEAR, 200 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, zoo.account_id);
        assert_eq!(candidate.amount, 200 * ONE_NEAR);

        // set bar's base stake amount to 100
        validator_pool.update_base_stake_amount(&bar.account_id, 100 * ONE_NEAR);
        bar = validator_pool
            .get_validator(&AccountId::new_unchecked("bar".to_string()))
            .unwrap();

        // reset staked amount
        foo.staked_amount = 100 * ONE_NEAR; // target is 100
        bar.staked_amount = 150 * ONE_NEAR; // target is 50
        zoo.staked_amount = 50 * ONE_NEAR; // target is 0
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // we have currently 300 already staked, 150 to unstake, target total 150,
        // the total stake amount is less than total base stake amount, satisfay base stake amount first,
        // thus bar is the most unbalanced one.

        let candidate = validator_pool.get_candidate_to_unstake(150 * ONE_NEAR, 150 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        // to avoid blocking unstake, more amount than the delta (100 NEAR) will be unstaked.
        assert_eq!(candidate.amount, 150 * ONE_NEAR);
        assert_eq!(candidate.validator.account_id, bar.account_id);
    }

    #[test]
    fn test_unstake_candidate_select_v2() {
        let mut validator_pool = ValidatorPool::new();

        let mut foo = validator_pool.add_validator(&AccountId::new_unchecked("foo".to_string()), 1);
        let mut bar = validator_pool.add_validator(&AccountId::new_unchecked("bar".to_string()), 1);
        let mut zoo = validator_pool.add_validator(&AccountId::new_unchecked("zoo".to_string()), 2);

        // manually set staked amounts
        foo.staked_amount = 170 * ONE_NEAR; // target is 100, delta is 70
        bar.staked_amount = 200 * ONE_NEAR; // target is 100, delta is 100
        zoo.staked_amount = 80 * ONE_NEAR; // target is 200, delta is -120
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // test step 1 only
        // foo's and bar's `delta` match the `total_amount_to_unstake`, and foo's is the smallest, so foo is selected
        let candidate = validator_pool.get_candidate_to_unstake_v2(50 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, foo.account_id);
        assert_eq!(candidate.amount, 50 * ONE_NEAR);

        // reset staked amounts
        foo.staked_amount = 110 * ONE_NEAR; // target is 100, delta is 10
        bar.staked_amount = 120 * ONE_NEAR; // target is 100, delta is 20
        zoo.staked_amount = 230 * ONE_NEAR; // target is 200, delta is 30
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // test step 2 with `amount_to_unstake = target / 2`
        // no `delta` is larger than `total_amount_to_unstake`, but bar has the largest `delta / target`, so bar is selected
        // unstake max(100 / 2, 20) = 50 NEAR from bar
        let candidate = validator_pool.get_candidate_to_unstake_v2(60 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, bar.account_id);
        assert_eq!(candidate.amount, 50 * ONE_NEAR);

        // reset staked amounts
        foo.staked_amount = 110 * ONE_NEAR; // target is 100, delta is 10
        bar.staked_amount = 120 * ONE_NEAR; // target is 100, delta is 20
        zoo.staked_amount = 410 * ONE_NEAR; // target is 200, delta is 210
        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // test step 2 with `amount_to_unstake = delta`
        // no `delta` is larger than `total_amount_to_unstake`, but zoo has the largest `delta / target`, so zoo is selected
        // unstake max(200 / 2, 210) = 210 NEAR from zoo
        let candidate = validator_pool.get_candidate_to_unstake_v2(240 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, zoo.account_id);
        assert_eq!(candidate.amount, 210 * ONE_NEAR);

        // add new validator with 0 weight
        let mut baz = validator_pool.add_validator(&AccountId::new_unchecked("baz".to_string()), 0);
        baz.staked_amount = 100 * ONE_NEAR; // target is 0, delta is 100
        validator_pool
            .validators
            .insert(&baz.account_id, &baz.clone().into());

        // test step 2 with zero target validator
        // no `delta` is larger than `total_amount_to_unstake`, but baz's `target` is zero, so baz is selected
        let candidate = validator_pool.get_candidate_to_unstake_v2(340 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, baz.account_id);
        assert_eq!(candidate.amount, 100 * ONE_NEAR);
    }

    #[test]
    fn test_unstake_candidate_select_v2_with_base_stake_amount() {
        let mut validator_pool = ValidatorPool::new();

        let mut foo = validator_pool.add_validator(&AccountId::new_unchecked("foo".to_string()), 1);
        let mut bar = validator_pool.add_validator(&AccountId::new_unchecked("bar".to_string()), 1);
        let mut zoo = validator_pool.add_validator(&AccountId::new_unchecked("zoo".to_string()), 1);

        // set foo's base stake amount to 200
        validator_pool.update_base_stake_amount(&foo.account_id, 200 * ONE_NEAR);
        foo = validator_pool
            .get_validator(&AccountId::new_unchecked("foo".to_string()))
            .unwrap();
        // set bar's base stake amount to 100
        validator_pool.update_base_stake_amount(&bar.account_id, 100 * ONE_NEAR);
        bar = validator_pool
            .get_validator(&AccountId::new_unchecked("bar".to_string()))
            .unwrap();
        // set zoo's base stake amount to 70
        validator_pool.update_base_stake_amount(&zoo.account_id, 70 * ONE_NEAR);
        zoo = validator_pool
            .get_validator(&AccountId::new_unchecked("zoo".to_string()))
            .unwrap();

        // manually set staked amount
        foo.staked_amount = 220 * ONE_NEAR; // target is 210, delta is 10, base is 200
        bar.staked_amount = 120 * ONE_NEAR; // target is 110, delta is 10, base is 100
        zoo.staked_amount = 90 * ONE_NEAR; // target is 80, delta is 10, base is 70

        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // zoo has largest `delta / target`, so it will be selected.
        // zoo's `target / 2` is 40 NEAR, `stake - base` is 20 NEAR, in order to
        // guarantee minimum stake amount, we can unstake no more than 20 NEAR
        let candidate = validator_pool.get_candidate_to_unstake_v2(30 * ONE_NEAR, 400 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, zoo.account_id);
        assert_eq!(candidate.amount, 20 * ONE_NEAR);
    }

    #[test]
    fn test_unstake_candidate_select_v2_with_total_staked_less_than_total_base() {
        let mut validator_pool = ValidatorPool::new();

        let mut foo = validator_pool.add_validator(&AccountId::new_unchecked("foo".to_string()), 1);
        let mut bar = validator_pool.add_validator(&AccountId::new_unchecked("bar".to_string()), 1);
        let mut zoo = validator_pool.add_validator(&AccountId::new_unchecked("zoo".to_string()), 1);

        // set foo's base stake amount to 200
        validator_pool.update_base_stake_amount(&foo.account_id, 200 * ONE_NEAR);
        foo = validator_pool
            .get_validator(&AccountId::new_unchecked("foo".to_string()))
            .unwrap();
        // set bar's base stake amount to 100
        validator_pool.update_base_stake_amount(&bar.account_id, 500 * ONE_NEAR);
        bar = validator_pool
            .get_validator(&AccountId::new_unchecked("bar".to_string()))
            .unwrap();
        // set zoo's base stake amount to 70
        validator_pool.update_base_stake_amount(&zoo.account_id, 300 * ONE_NEAR);
        zoo = validator_pool
            .get_validator(&AccountId::new_unchecked("zoo".to_string()))
            .unwrap();

        // manually set staked amount
        foo.staked_amount = 220 * ONE_NEAR; // target is 165, delta is 55, base is 200
        bar.staked_amount = 300 * ONE_NEAR; // target is 225, delta is 75, base is 500
        zoo.staked_amount = 230 * ONE_NEAR; // target is 172.5, delta is 57.5, base is 300

        validator_pool
            .validators
            .insert(&foo.account_id, &foo.clone().into());
        validator_pool
            .validators
            .insert(&bar.account_id, &bar.clone().into());
        validator_pool
            .validators
            .insert(&zoo.account_id, &zoo.clone().into());

        // foo has the smallest delta that match the `total_amount_to_unstake`, it will be selected.
        // the `staked_amount - base_stake_amount` of foo is 20 NEAR, but actually unstaked amount is 30 NEAR
        // because `total_staked_near_amount` is less than `total_base_stake_amount`
        let candidate = validator_pool.get_candidate_to_unstake_v2(30 * ONE_NEAR, 720 * ONE_NEAR);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, foo.account_id);
        assert_eq!(candidate.amount, 30 * ONE_NEAR);
    }

    #[test]
    fn test_unstake_candidate_select_v2_that_selects_min_delta() {
        let mut validator_pool = ValidatorPool::new();

        let mut v1 = validator_pool.add_validator(&AccountId::new_unchecked("v1".to_string()), 12);
        let mut v2 = validator_pool.add_validator(&AccountId::new_unchecked("v2".to_string()), 11);
        let mut v3 = validator_pool.add_validator(&AccountId::new_unchecked("v3".to_string()), 3);
        let mut v4 = validator_pool.add_validator(&AccountId::new_unchecked("v4".to_string()), 7);
        let mut v5 = validator_pool.add_validator(&AccountId::new_unchecked("v5".to_string()), 7);
        let mut v6 = validator_pool.add_validator(&AccountId::new_unchecked("v6".to_string()), 5);
        let mut v7 = validator_pool.add_validator(&AccountId::new_unchecked("v7".to_string()), 5);
        let mut v8 = validator_pool.add_validator(&AccountId::new_unchecked("v8".to_string()), 6);
        let mut v9 = validator_pool.add_validator(&AccountId::new_unchecked("v9".to_string()), 8);
        let mut v10 = validator_pool.add_validator(&AccountId::new_unchecked("v10".to_string()), 8);

        // manually set staked amounts
        v1.staked_amount = 3950 * ONE_NEAR; // target ≈ 3870,    delta ≈ 80
        v2.staked_amount = 3600 * ONE_NEAR; // target ≈ 3547.5,  delta ≈ 52.5
        v3.staked_amount = 1000 * ONE_NEAR; // target ≈ 967.5,   delta ≈ 32.5
        v4.staked_amount = 4000 * ONE_NEAR; // target ≈ 2257.5,  delta ≈ 1742.5
        v5.staked_amount = 1100 * ONE_NEAR; // target ≈ 2257.5,  delta ≈ -1157.5
        v6.staked_amount = 2500 * ONE_NEAR; // target ≈ 1612.5,  delta ≈ 887.5
        v7.staked_amount = 1700 * ONE_NEAR; // target ≈ 1612.5,  delta ≈ 87.5
        v8.staked_amount = 1700 * ONE_NEAR; // target ≈ 1935,    delta ≈ -235
        v9.staked_amount = 2500 * ONE_NEAR; // target ≈ 2580,    delta ≈ -80
        v10.staked_amount = 1200 * ONE_NEAR; // target ≈ 3333.33, delta ≈ -2133.33

        // set v3 pending release
        v3.unstake_fired_epoch = 9;

        validator_pool
            .validators
            .insert(&v1.account_id, &v1.clone().into());
        validator_pool
            .validators
            .insert(&v2.account_id, &v2.clone().into());
        validator_pool
            .validators
            .insert(&v3.account_id, &v3.clone().into());
        validator_pool
            .validators
            .insert(&v4.account_id, &v4.clone().into());
        validator_pool
            .validators
            .insert(&v5.account_id, &v5.clone().into());
        validator_pool
            .validators
            .insert(&v6.account_id, &v6.clone().into());
        validator_pool
            .validators
            .insert(&v7.account_id, &v7.clone().into());
        validator_pool
            .validators
            .insert(&v8.account_id, &v8.clone().into());
        validator_pool
            .validators
            .insert(&v9.account_id, &v9.clone().into());
        validator_pool
            .validators
            .insert(&v10.account_id, &v10.clone().into());

        let total_amount_to_unstake = 30 * ONE_NEAR;
        let total_staked_near_amount = 23220 * ONE_NEAR;

        // v2 should be selected because it has min `delta` that satisfy the unstake amount
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v2.account_id);
        let unstaked_amount = 30 * ONE_NEAR;
        assert_eq!(candidate.amount, unstaked_amount);
    }

    #[test]
    fn test_unstake_candidate_select_v2_that_selects_largest_ratio_of_delta_to_target() {
        let mut validator_pool = ValidatorPool::new();

        let mut v1 = validator_pool.add_validator(&AccountId::new_unchecked("v1".to_string()), 8);
        let mut v2 = validator_pool.add_validator(&AccountId::new_unchecked("v2".to_string()), 11);
        let mut v3 = validator_pool.add_validator(&AccountId::new_unchecked("v3".to_string()), 0);
        let mut v4 = validator_pool.add_validator(&AccountId::new_unchecked("v4".to_string()), 7);
        let mut v5 = validator_pool.add_validator(&AccountId::new_unchecked("v5".to_string()), 5);
        let mut v6 = validator_pool.add_validator(&AccountId::new_unchecked("v6".to_string()), 7);
        let mut v7 = validator_pool.add_validator(&AccountId::new_unchecked("v7".to_string()), 0);
        let mut v8 = validator_pool.add_validator(&AccountId::new_unchecked("v8".to_string()), 6);
        let mut v9 = validator_pool.add_validator(&AccountId::new_unchecked("v9".to_string()), 8);
        let mut v10 = validator_pool.add_validator(&AccountId::new_unchecked("v10".to_string()), 8);

        // manually set staked amounts
        v1.staked_amount = 3000 * ONE_NEAR; // target ≈ 1546.67, delta ≈ 1453.33
        v2.staked_amount = 4000 * ONE_NEAR; // target ≈ 2126.67, delta ≈ 1873.33
        v3.staked_amount = 100 * ONE_NEAR; // target ≈ 0,  delta ≈ 100
        v4.staked_amount = 300 * ONE_NEAR; // target ≈ 1353.33, delta ≈ -1053.33
        v5.staked_amount = 1500 * ONE_NEAR; // target ≈ 966.67,       delta ≈ 533.33
        v6.staked_amount = 2000 * ONE_NEAR; // target ≈ 1353.33,  delta ≈ 646.67
        v7.staked_amount = 3000 * ONE_NEAR; // target ≈ 0,  delta ≈ 3000
        v8.staked_amount = 700 * ONE_NEAR; // target ≈ 1160, delta ≈ -460
        v9.staked_amount = 800 * ONE_NEAR; // target ≈ 1546.67, delta ≈ -746.67
        v10.staked_amount = 1200 * ONE_NEAR; // target ≈ 1546.67, delta ≈ -346.67

        validator_pool
            .validators
            .insert(&v1.account_id, &v1.clone().into());
        validator_pool
            .validators
            .insert(&v2.account_id, &v2.clone().into());
        validator_pool
            .validators
            .insert(&v3.account_id, &v3.clone().into());
        validator_pool
            .validators
            .insert(&v4.account_id, &v4.clone().into());
        validator_pool
            .validators
            .insert(&v5.account_id, &v5.clone().into());
        validator_pool
            .validators
            .insert(&v6.account_id, &v6.clone().into());
        validator_pool
            .validators
            .insert(&v7.account_id, &v7.clone().into());
        validator_pool
            .validators
            .insert(&v8.account_id, &v8.clone().into());
        validator_pool
            .validators
            .insert(&v9.account_id, &v9.clone().into());
        validator_pool
            .validators
            .insert(&v10.account_id, &v10.clone().into());

        // init params
        let mut total_amount_to_unstake = 5000 * ONE_NEAR;
        let total_staked_near_amount = 11600 * ONE_NEAR;

        // unstake total 5000 NEAR
        // v7 should be selected because its `target` is 0 and has largest `delta`
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v7.account_id);
        let unstaked_amount = 3000 * ONE_NEAR;
        assert_eq!(candidate.amount, unstaked_amount);

        // 1st unstake 3000 NEAR from v7
        total_amount_to_unstake -= unstaked_amount;
        v7.staked_amount -= unstaked_amount; // target ≈ 0,  delta ≈ 0
        validator_pool
            .validators
            .insert(&v7.account_id, &v7.clone().into());

        // 2000 NEAR needs to be unstaked
        // v3 should be selected because its `target` is 0 and has largest `delta`
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v3.account_id);
        let unstaked_amount = 100 * ONE_NEAR;
        assert_eq!(candidate.amount, unstaked_amount);

        // 2nd unstake 100 NEAR from v3
        total_amount_to_unstake -= unstaked_amount;
        v3.staked_amount -= unstaked_amount; // target ≈ 0,  delta ≈ 0
        validator_pool
            .validators
            .insert(&v3.account_id, &v3.clone().into());

        // 1900 NEAR needs to be unstaked
        // v1 should be selected because it has largest `delta / target`
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v1.account_id);
        let unstaked_amount = 1453333333333333333333333334_u128;
        assert_eq!(candidate.amount, unstaked_amount);

        // 4th unstake around 1453.33 NEAR from v1
        total_amount_to_unstake -= unstaked_amount;
        v1.staked_amount -= unstaked_amount;
        validator_pool
            .validators
            .insert(&v1.account_id, &v1.clone().into());

        // around 446.67 NEAR needs to be unstaked
        // v5 should be selected because it has min `delta` that satisfy the unstake amount
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v5.account_id);
        let unstaked_amount = 446666666666666666666666666_u128;
        assert_eq!(candidate.amount, unstaked_amount);
    }

    #[test]
    fn test_unstake_candidate_select_v2_that_unstakes_half_target() {
        let mut validator_pool = ValidatorPool::new();

        let mut v1 = validator_pool.add_validator(&AccountId::new_unchecked("v1".to_string()), 12);
        let mut v2 = validator_pool.add_validator(&AccountId::new_unchecked("v2".to_string()), 11);
        let mut v3 = validator_pool.add_validator(&AccountId::new_unchecked("v3".to_string()), 5);
        let mut v4 = validator_pool.add_validator(&AccountId::new_unchecked("v4".to_string()), 7);
        let mut v5 = validator_pool.add_validator(&AccountId::new_unchecked("v5".to_string()), 8);
        let mut v6 = validator_pool.add_validator(&AccountId::new_unchecked("v6".to_string()), 7);
        let mut v7 = validator_pool.add_validator(&AccountId::new_unchecked("v7".to_string()), 6);
        let mut v8 = validator_pool.add_validator(&AccountId::new_unchecked("v8".to_string()), 6);
        let mut v9 = validator_pool.add_validator(&AccountId::new_unchecked("v9".to_string()), 8);
        let mut v10 = validator_pool.add_validator(&AccountId::new_unchecked("v10".to_string()), 8);

        // manually set staked amounts
        v1.staked_amount = 2000 * ONE_NEAR; // target ≈ 1661.54, delta ≈ 338.46
        v2.staked_amount = 2800 * ONE_NEAR; // target ≈ 1523.08, delta ≈ 1276.92
        v3.staked_amount = 1000 * ONE_NEAR; // target ≈ 692.31,  delta ≈ 307.69
        v4.staked_amount = 300 * ONE_NEAR; // target ≈ 969.23, delta ≈ -669.23
        v5.staked_amount = 1500 * ONE_NEAR; // target ≈ 1107.69,       delta ≈ 392.31
        v6.staked_amount = 1300 * ONE_NEAR; // target ≈ 969.23,  delta ≈ 330.77
        v7.staked_amount = 1200 * ONE_NEAR; // target ≈ 830.77,  delta ≈ 369.23
        v8.staked_amount = 700 * ONE_NEAR; // target ≈ 830.77, delta ≈ -130.77
        v9.staked_amount = 800 * ONE_NEAR; // target ≈ 1107.69, delta ≈ -307.69
        v10.staked_amount = 1200 * ONE_NEAR; // target ≈ 1107.69, delta ≈ 92.31

        validator_pool
            .validators
            .insert(&v1.account_id, &v1.clone().into());
        validator_pool
            .validators
            .insert(&v2.account_id, &v2.clone().into());
        validator_pool
            .validators
            .insert(&v3.account_id, &v3.clone().into());
        validator_pool
            .validators
            .insert(&v4.account_id, &v4.clone().into());
        validator_pool
            .validators
            .insert(&v5.account_id, &v5.clone().into());
        validator_pool
            .validators
            .insert(&v6.account_id, &v6.clone().into());
        validator_pool
            .validators
            .insert(&v7.account_id, &v7.clone().into());
        validator_pool
            .validators
            .insert(&v8.account_id, &v8.clone().into());
        validator_pool
            .validators
            .insert(&v9.account_id, &v9.clone().into());
        validator_pool
            .validators
            .insert(&v10.account_id, &v10.clone().into());

        // init params
        let mut total_amount_to_unstake = 2000 * ONE_NEAR;
        let total_staked_near_amount = 10800 * ONE_NEAR;

        // unstake total 2000 NEAR
        // v2 should be selected because it has largest `delta / target`
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v2.account_id);
        let unstaked_amount = 1276923076923076923076923077;
        assert_eq!(candidate.amount, unstaked_amount);

        // 1st unstake around 1276.92 NEAR from v2
        total_amount_to_unstake -= unstaked_amount;
        v2.staked_amount -= unstaked_amount; // target ≈ 1523.08,  delta ≈ 0
        validator_pool
            .validators
            .insert(&v2.account_id, &v2.clone().into());

        // 723.08 NEAR needs to be unstaked
        // v7 should be selected because it has largest `delta / target`
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v7.account_id);
        let unstaked_amount = 415384615384615384615384615_u128;
        assert_eq!(candidate.amount, unstaked_amount);

        // 2nd unstake around 415.38 NEAR from v7
        total_amount_to_unstake -= unstaked_amount;
        v7.staked_amount -= unstaked_amount; // target ≈ 830.77,  delta ≈ -46.15
        validator_pool
            .validators
            .insert(&v7.account_id, &v7.clone().into());

        // around 307.69 NEAR needs to be unstaked
        // v3 should be selected because it has min `delta` that satisfy the unstake amount
        let candidate = validator_pool
            .get_candidate_to_unstake_v2(total_amount_to_unstake, total_staked_near_amount);
        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.validator.account_id, v3.account_id);
        let unstaked_amount = 307692307692307692307692308_u128;
        assert_eq!(candidate.amount, unstaked_amount);
    }
}
