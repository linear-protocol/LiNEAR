//! This module contains all contract state versions, which are needed
//! when upgrading contract.
use crate::account::Account;
use crate::validator_pool::{Validator, VersionedValidator};
use crate::{types::*, Fraction};
// use crate::StorageKey;
use crate::ValidatorPool;
use near_sdk::json_types::U128;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap, UnorderedSet, Vector},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, Balance, EpochHeight, StorageUsage, Timestamp,
};

/// Changes to root state in v1.6.0:
/// - removed liquidity_pool
/// - removed staking farms fields: farms, active_farms, authorized_farm_tokens
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractV1_6_0 {
    /// The account ID of the owner
    owner_id: AccountId,
    /// The accounts that are able to change key parameters and settings in the contract such as validator pool membership
    managers: UnorderedSet<AccountId>,
    /// The account ID of the treasury that manages portion of the received fees and rewards.
    treasury_id: AccountId,
    /// Total amount of LiNEAR that was minted (minus burned).
    total_share_amount: ShareBalance,
    /// Total amount of NEAR that was staked by users to this contract.         
    ///
    /// This is effectively 1) amount of NEAR that was deposited to this contract but hasn't yet been staked on any validators
    /// plus 2) amount of NEAR that has already been staked on validators.    
    /// Note that the amount of NEAR that is pending release or is already released by hasn't been withdrawn is not considered.
    total_staked_near_amount: Balance,
    /// Persistent map from an account ID to the corresponding account.
    accounts: UnorderedMap<AccountId, Account>,
    /// Pause the contract for maintenance, all user interactions are stopped. Only the owner can perform pause and resume.
    /// It doesn't affect the staking shares or reward distribution.
    /// The contract is not paused by default.
    paused: bool,

    /// The storage size in bytes for one account.
    account_storage_usage: StorageUsage,

    /// Beneficiaries for staking rewards.
    beneficiaries: UnorderedMap<AccountId, u32>,

    // --- Validator Pool ---
    /// The validator pool that manage the actions against validators
    validator_pool: ValidatorPool,
    /// The whitelist contract ID, which controls the staking pool whitelist.
    whitelist_account_id: Option<AccountId>,
    /// Amount of NEAR that is requested to stake by all users during the last epoch
    epoch_requested_stake_amount: Balance,
    /// Amount of NEAR that is requested to unstake by all users during the last epoch
    epoch_requested_unstake_amount: Balance,

    /// Amount of NEAR that needs to be settled by staking on validators
    stake_amount_to_settle: Balance,
    /// Amount of NEAR that needs to be settled by unstaking from validators
    unstake_amount_to_settle: Balance,
    /// Last epoch height stake/unstake settlements were calculated
    last_settlement_epoch: EpochHeight,
}

/// The ValidatorPool struct has no change in v1.4.0 since v1.3.0
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidatorPoolV1_4_0 {
    pub validators: UnorderedMap<AccountId, VersionedValidator>,
    pub total_weight: u16,
    pub total_base_stake_amount: Balance,
}

/// The Validator struct added `draining` in v1.4.0
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorV1_4_0 {
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

impl From<ValidatorV1_4_0> for Validator {
    fn from(v: ValidatorV1_4_0) -> Self {
        Validator {
            account_id: v.account_id,
            weight: v.weight,
            staked_amount: v.staked_amount,
            unstaked_amount: v.unstaked_amount,
            base_stake_amount: v.base_stake_amount,
            unstake_fired_epoch: v.unstake_fired_epoch,
            last_unstake_fired_epoch: v.last_unstake_fired_epoch,
            draining: v.draining,
            executing: false,
        }
    }
}

/// There's no root state change in v1.1.0 to v1.5.0
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractV1_3_0 {
    /// The account ID of the owner
    pub owner_id: AccountId,
    /// The accounts that are able to change key parameters and settings in the contract such as validator pool membership
    pub managers: UnorderedSet<AccountId>,
    /// The account ID of the treasury that manages portion of the received fees and rewards.
    pub treasury_id: AccountId,
    /// Total amount of LiNEAR that was minted (minus burned).
    pub total_share_amount: ShareBalance,
    /// Total amount of NEAR that was staked by users to this contract.
    ///
    /// This is effectively 1) amount of NEAR that was deposited to this contract but hasn't yet been staked on any validators
    /// plus 2) amount of NEAR that has already been staked on validators.
    /// Note that the amount of NEAR that is pending release or is already released by hasn't been withdrawn is not considered.
    pub total_staked_near_amount: Balance,
    /// Persistent map from an account ID to the corresponding account.
    pub accounts: UnorderedMap<AccountId, Account>,
    /// Pause the contract for maintenance, all user interactions are stopped. Only the owner can perform pause and resume.
    /// It doesn't affect the staking shares or reward distribution.
    /// The contract is not paused by default.
    pub paused: bool,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,

    /// Beneficiaries for staking rewards.
    pub beneficiaries: UnorderedMap<AccountId, u32>,

    /// The single-direction liquidity pool that enables instant unstake
    pub liquidity_pool: LiquidityPool,

    // --- Validator Pool ---
    /// The validator pool that manage the actions against validators
    pub validator_pool: ValidatorPool,
    /// The whitelist contract ID, which controls the staking pool whitelist.
    pub whitelist_account_id: Option<AccountId>,
    /// Amount of NEAR that is requested to stake by all users during the last epoch
    pub epoch_requested_stake_amount: Balance,
    /// Amount of NEAR that is requested to unstake by all users during the last epoch
    pub epoch_requested_unstake_amount: Balance,

    /// Amount of NEAR that needs to be settled by staking on validators
    pub stake_amount_to_settle: Balance,
    /// Amount of NEAR that needs to be settled by unstaking from validators
    pub unstake_amount_to_settle: Balance,
    /// Last epoch height stake/unstake settlements were calculated
    pub last_settlement_epoch: EpochHeight,

    // --- Staking Farm ---
    /// Farm tokens.
    pub farms: Vector<Farm>,
    /// Active farms: indicies into `farms`.
    pub active_farms: Vec<u64>,
    /// Authorized users, allowed to add farms.
    /// This is done to prevent farm spam with random tokens.
    /// Should not be a large number.
    // authorized_users: UnorderedSet<AccountId>,
    /// Authorized tokens for farms.
    /// Required because any contract can call method with ft_transfer_call, so must verify that contract will accept it.
    pub authorized_farm_tokens: UnorderedSet<AccountId>,
}
/// The ValidatorPool struct added `total_base_stake_amount` in v1.3.0
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidatorPoolV1_3_0 {
    pub validators: UnorderedMap<AccountId, Validator>,
    pub total_weight: u16,
    pub total_base_stake_amount: Balance,
}
/// The Validator struct added `base_stake_amount` in v1.3.0
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorV1_3_0 {
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
}

impl From<ValidatorV1_3_0> for Validator {
    fn from(v: ValidatorV1_3_0) -> Self {
        Validator {
            account_id: v.account_id,
            weight: v.weight,
            staked_amount: v.staked_amount,
            unstaked_amount: v.unstaked_amount,
            base_stake_amount: v.base_stake_amount,
            unstake_fired_epoch: v.unstake_fired_epoch,
            last_unstake_fired_epoch: v.last_unstake_fired_epoch,
            draining: false,
            executing: false,
        }
    }
}

/// There's no any state change in v1.2.0, but it retired built-in liquidity pool
/// ContractV1_2_0

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ContractV1_1_0 {
    /// The account ID of the owner who's running the liquid staking contract.
    pub owner_id: AccountId,
    /// The accounts that are able to change key parameters and settings in the contract such as validator pool membership
    pub managers: UnorderedSet<AccountId>,
    /// The account ID of the treasury that manages portion of the received fees and rewards.
    pub treasury_id: AccountId,
    /// Total amount of LiNEAR that was minted (minus burned).
    pub total_share_amount: ShareBalance,
    /// Total amount of NEAR that was staked by users to this contract.         
    ///
    /// This is effectively 1) amount of NEAR that was deposited to this contract but hasn't yet been staked on any validators
    /// plus 2) amount of NEAR that has already been staked on validators.    
    /// Note that the amount of NEAR that is pending release or is already released by hasn't been withdrawn is not considered.
    pub total_staked_near_amount: Balance,
    /// Persistent map from an account ID to the corresponding account.
    pub accounts: UnorderedMap<AccountId, Account>,
    /// Whether the staking is paused.
    /// When paused, the account unstakes everything (stakes 0) and doesn't restake.
    /// It doesn't affect the staking shares or reward distribution.
    /// Pausing is useful for node maintenance. Only the owner can pause and resume staking.
    /// The contract is not paused by default.
    pub paused: bool,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,

    /// Beneficiaries for staking rewards.
    pub beneficiaries: UnorderedMap<AccountId, u32>,

    /// The single-direction liquidity pool that enables instant unstake
    pub liquidity_pool: LiquidityPool,

    // --- Validator Pool ---
    /// The validator pool that manage the actions against validators
    pub validator_pool: ValidatorPoolV1_0_0,
    /// The whitelist contract ID, which controls the staking pool whitelist.
    pub whitelist_account_id: Option<AccountId>,
    /// Amount of NEAR that is requested to stake by all users during the last epoch
    pub epoch_requested_stake_amount: Balance,
    /// Amount of NEAR that is requested to unstake by all users during the last epoch
    pub epoch_requested_unstake_amount: Balance,

    /// Amount of NEAR that needs to be settled by staking on validators
    pub stake_amount_to_settle: Balance,
    /// Amount of NEAR that needs to be settled by unstaking from validators
    pub unstake_amount_to_settle: Balance,
    /// Last epoch height stake/unstake settlements were calculated
    pub last_settlement_epoch: EpochHeight,

    // --- Staking Farm ---
    /// Farm tokens.
    pub farms: Vector<Farm>,
    /// Active farms: indicies into `farms`.
    pub active_farms: Vec<u64>,
    /// Authorized users, allowed to add farms.
    /// This is done to prevent farm spam with random tokens.
    /// Should not be a large number.
    // authorized_users: UnorderedSet<AccountId>,
    /// Authorized tokens for farms.
    /// Required because any contract can call method with ft_transfer_call, so must verify that contract will accept it.
    pub authorized_farm_tokens: UnorderedSet<AccountId>,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ContractV1_0_0 {
    /// The account ID of the owner who's running the liquid staking contract.
    pub owner_id: AccountId,
    /// The accounts that are able to change key parameters and settings in the contract such as validator pool membership
    pub managers: UnorderedSet<AccountId>,
    /// The account ID of the treasury that manages portion of the received fees and rewards.
    pub treasury_id: AccountId,
    /// Total amount of LiNEAR that was minted (minus burned).
    pub total_share_amount: ShareBalance,
    /// Total amount of NEAR that was staked by users to this contract.         
    ///
    /// This is effectively 1) amount of NEAR that was deposited to this contract but hasn't yet been staked on any validators
    /// plus 2) amount of NEAR that has already been staked on validators.    
    /// Note that the amount of NEAR that is pending release or is already released by hasn't been withdrawn is not considered.
    pub total_staked_near_amount: Balance,
    /// Persistent map from an account ID to the corresponding account.
    pub accounts: UnorderedMap<AccountId, Account>,
    /// Whether the staking is paused.
    /// When paused, the account unstakes everything (stakes 0) and doesn't restake.
    /// It doesn't affect the staking shares or reward distribution.
    /// Pausing is useful for contract maintenance. Only the owner can pause and resume staking.
    /// The contract is not paused by default.
    pub paused: bool,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,

    /// Beneficiaries for staking rewards.
    pub beneficiaries: UnorderedMap<AccountId, Fraction>,

    /// The single-direction liquidity pool that enables instant unstake
    pub liquidity_pool: LiquidityPool,

    // --- Validator Pool ---
    /// The validator pool that manage the actions against validators
    pub validator_pool: ValidatorPoolV1_0_0,
    /// Amount of NEAR that is requested to stake by all users during the last epoch
    pub epoch_requested_stake_amount: Balance,
    /// Amount of NEAR that is requested to unstake by all users during the last epoch
    pub epoch_requested_unstake_amount: Balance,

    /// Amount of NEAR that needs to be settled by staking on validators
    pub stake_amount_to_settle: Balance,
    /// Amount of NEAR that needs to be settled by unstaking from validators
    pub unstake_amount_to_settle: Balance,
    /// Last epoch height stake/unstake settlements were calculated
    pub last_settlement_epoch: EpochHeight,

    // --- Staking Farm ---
    /// Farm tokens.
    pub farms: Vector<Farm>,
    /// Active farms: indicies into `farms`.
    pub active_farms: Vec<u64>,
    /// Authorized users, allowed to add farms.
    /// This is done to prevent farm spam with random tokens.
    /// Should not be a large number.
    // authorized_users: UnorderedSet<AccountId>,
    /// Authorized tokens for farms.
    /// Required because any contract can call method with ft_transfer_call, so must verify that contract will accept it.
    pub authorized_farm_tokens: UnorderedSet<AccountId>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorV1_0_0 {
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

impl From<ValidatorV1_0_0> for Validator {
    fn from(v: ValidatorV1_0_0) -> Self {
        Validator {
            account_id: v.account_id,
            weight: v.weight,
            staked_amount: v.staked_amount,
            unstaked_amount: v.unstaked_amount,
            base_stake_amount: 0,
            unstake_fired_epoch: v.unstake_fired_epoch,
            last_unstake_fired_epoch: v.last_unstake_fired_epoch,
            draining: false,
            executing: false,
        }
    }
}

/// A pool of validators.
/// The main function of this struct is to
/// store validator info and calculate the best candidate to stake/unstake.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidatorPoolV1_0_0 {
    validators: UnorderedMap<AccountId, ValidatorV1_0_0>,
    total_weight: u16,
}

// --- ValidatorPool state migration --
// Used in v1.3.0 upgrade
// impl ValidatorPoolV1_0_0 {
//     pub fn migrate(&mut self) -> ValidatorPool {
//         // migrate old validators into the new structure
//         let mut new_validators: UnorderedMap<AccountId, VValidator> =
//             UnorderedMap::new(StorageKey::ValidatorsV1);
//         let old_validators = self.validators.values_as_vector();
//         for v in old_validators.iter() {
//             new_validators.insert(&v.account_id.clone(), &v.into_validator().into());
//         }

//         // remove old map
//         self.validators.clear();

//         ValidatorPool {
//             validators: new_validators,
//             total_weight: self.total_weight,
//             total_base_stake_amount: 0,
//         }
//     }
// }

// Liquidity Pool legacy structs, removed since v1.6.0

#[derive(BorshSerialize, BorshDeserialize)]
pub struct LiquidityPool {
    /// List of tokens in the pool
    pub token_account_ids: Vec<AccountId>,
    /// How much token in the pool
    pub amounts: Vec<Balance>,
    /// Shares of the pool by liquidity providers.
    pub shares: LookupMap<AccountId, Balance>,
    /// Total number of shares
    pub shares_total_supply: Balance,

    /// Configuration of the pool
    pub config: LiquidityPoolConfig,

    /// Total swap fee in LiNEAR received by the pool
    pub total_fee_shares: ShareBalance,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidityPoolConfig {
    /// The expected near amount used in the fee calculation formula.
    /// If the NEAR amount in the liquidity pool exceeds the expectation, the
    /// swap fee will be the `min_fee_bps`
    pub expected_near_amount: U128,
    /// Max fee in basis points
    pub max_fee_bps: u32,
    /// Min fee in basis points
    pub min_fee_bps: u32,
    /// Fee allocated to treasury in basis points
    pub treasury_fee_bps: u32,
}

// Staking Farm legacy structs, removed since v1.6.0

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct RewardDistribution {
    pub undistributed: Balance,
    pub unclaimed: Balance,
    pub reward_per_share: U256,
    pub reward_round: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Farm {
    pub name: String,
    pub token_id: AccountId,
    pub amount: Balance,
    pub start_date: Timestamp,
    pub end_date: Timestamp,
    pub last_distribution: RewardDistribution,
}
