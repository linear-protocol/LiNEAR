//! This module contains all contract state versions, which are needed
//! when upgrading contract.
use crate::account::Account;
use crate::types::*;
use crate::Farm;
use crate::Fraction;
use crate::LiquidityPool;
use crate::ValidatorPool;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{UnorderedMap, UnorderedSet, Vector},
    near_bindgen, AccountId, Balance, EpochHeight, StorageUsage,
};

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
    pub validator_pool: ValidatorPool,
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
