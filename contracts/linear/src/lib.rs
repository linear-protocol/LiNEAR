#![allow(deprecated)]

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{UnorderedMap, UnorderedSet},
    env, ext_contract,
    json_types::U128,
    near_bindgen, require,
    serde::{Deserialize, Serialize},
    AccountId, Balance, BorshStorageKey, EpochHeight, PanicOnDefault, PublicKey, StorageUsage,
};

mod account;
mod epoch_actions;
mod errors;
mod events;
mod fungible_token;
mod internal;
mod legacy;
mod metadata;
mod owner;
mod stake;
mod types;
mod upgrade;
mod utils;
mod validator_pool;
mod view;

use crate::account::*;
use crate::errors::*;
use crate::fungible_token::*;
use crate::types::*;
use crate::utils::*;
use crate::validator_pool::*;

/// ONLY APPEND to this list for new variants
#[allow(dead_code)]
#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    #[deprecated(since = "1.6.0", note = "removed built-in liquidity pool")]
    Shares,
    Beneficiaries,
    #[deprecated(since = "1.3.0", note = "replaced by ValidatorsV1")]
    Validators, // ValidatorsV0 (Don't comment out this)
    #[deprecated(since = "1.6.0", note = "removed staking farm")]
    Farms,
    #[deprecated(since = "1.6.0", note = "removed staking farm")]
    AuthorizedFarmTokens,
    Managers,
    ValidatorsV1, // Used in v1.3.0 upgrade
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct LiquidStakingContract {
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

#[near_bindgen]
impl LiquidStakingContract {
    /// Initializes the contract with the given owner_id.
    ///
    /// The entire current balance of this contract will be used to stake. This allows contract to
    /// always maintain staking shares that can't be unstaked or withdrawn.
    /// It prevents inflating the price of the share too much.
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        require!(!env::state_exists(), ERR_ALREADY_INITIALZED);
        require!(
            env::account_locked_balance() == 0,
            ERR_ACCOUNT_STAKING_WHILE_INIT
        );

        let account_balance = env::account_balance();
        // 20 NEAR is required to init this contract,
        // 10 will be used as init staking, 10 will be left for storage
        require!(
            account_balance >= 20 * ONE_NEAR,
            format!(
                "{}. required: {}",
                ERR_NO_ENOUGH_INIT_DEPOSIT,
                20 * ONE_NEAR
            )
        );
        let mut this = Self {
            owner_id: owner_id.clone(),
            managers: UnorderedSet::new(StorageKey::Managers),
            treasury_id: owner_id.clone(),
            total_share_amount: 10 * ONE_NEAR,
            total_staked_near_amount: 10 * ONE_NEAR,
            accounts: UnorderedMap::new(StorageKey::Accounts),
            paused: false,
            account_storage_usage: 0,
            beneficiaries: UnorderedMap::new(StorageKey::Beneficiaries),
            // Validator Pool
            validator_pool: ValidatorPool::new(),
            whitelist_account_id: None,
            epoch_requested_stake_amount: 10 * ONE_NEAR,
            epoch_requested_unstake_amount: 0,
            stake_amount_to_settle: 0,
            unstake_amount_to_settle: 0,
            last_settlement_epoch: 0,
        };
        this.internal_add_manager(&owner_id);
        this.measure_account_storage_usage();
        this
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn measure_account_storage_usage(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = AccountId::new_unchecked("a".repeat(64));
        self.accounts.insert(&tmp_account_id, &Account::default());
        self.account_storage_usage = env::storage_usage() - initial_storage_usage;
        self.accounts.remove(&tmp_account_id);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        // let contract = LiquidStakingContract::new();
        testing_env!(context.is_view(true).build());
    }
}
