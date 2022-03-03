use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize,},
    serde::{Deserialize, Serialize,},
    json_types::{U128},
    collections::{UnorderedMap},
    env, near_bindgen, ext_contract, require,
    AccountId, Balance, PanicOnDefault, EpochHeight, PublicKey, StorageUsage
};

mod view;
mod types;
mod utils;
mod owner;
mod events;
mod errors;
mod account;
mod internal;
mod staking_pool;
mod epoch_actions;
mod fungible_token_core;
mod fungible_token_metadata;
mod fungible_token_storage;
mod fungible_token_custom;
mod liquidity_pool;

use crate::types::*;
use crate::utils::*;
use crate::errors::*;
use crate::account::*;
use crate::staking_pool::*;
// use crate::internal::*;
pub use crate::fungible_token_core::*;
pub use crate::fungible_token_metadata::*;
pub use crate::fungible_token_storage::*;
pub use crate::fungible_token_custom::*;
pub use crate::liquidity_pool::*;


/// Interface for the contract itself.
#[ext_contract(ext_self)]
pub trait SelfContract {
    /// A callback to check the result of the staking action.
    /// In case the stake amount is less than the minimum staking threshold, the staking action
    /// fails, and the stake amount is not changed. This might lead to inconsistent state and the
    /// follow withdraw calls might fail. To mitigate this, the contract will issue a new unstaking
    /// action in case of the failure of the first staking action.
    fn on_stake_action(&mut self);
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Fraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl Fraction {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        let f = Self {
            numerator,
            denominator,
        };
        f.assert_valid();
        return f;
    }

    pub fn assert_valid(&self) {
        require!(
            self.denominator != 0,
            ERR_FRACTION_BAD_DENOMINATOR
        );
        require!(
            self.numerator <= self.denominator,
            ERR_FRACTION_BAD_NUMERATOR
        );
    }

    pub fn multiply(&self, value: u128) -> u128 {
        (U256::from(self.numerator) * U256::from(value) / U256::from(self.denominator)).as_u128()
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct LiquidStakingContract {
    /// The account ID of the owner who's running the staking validator node.
    /// NOTE: This is different from the current account ID which is used as a validator account.
    /// The owner of the staking pool can change staking public key and adjust reward fees.
    pub owner_id: AccountId,
    /// The last epoch height when `ping` was called.
    pub last_epoch_height: EpochHeight,
    /// The last total balance of the account (consists of staked and unstaked balances).
    pub last_total_balance: Balance,
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

    /// The validator pool that manage the actions against validators
    validator_pool: ValidatorPool,
    /// Amount of NEAR that is requested to stake by all users during the last epoch
    epoch_requested_stake_amount: Balance,
    /// Amount of NEAR that is requested to unstake by all users during the last epoch
    epoch_requested_unstake_amount: Balance,

    /// The single-direction liquidity pool that enables instant unstake
    liquidity_pool: LiquidityPool,
  
    /// Beneficiaries for staking rewards.
    beneficiaries: UnorderedMap<AccountId, Fraction>,
}

#[near_bindgen]
impl LiquidStakingContract {
    /// Initializes the contract with the given owner_id and initial reward fee fraction that 
    /// owner charges for the validation work.
    ///
    /// The entire current balance of this contract will be used to stake. This allows contract to
    /// always maintain staking shares that can't be unstaked or withdrawn.
    /// It prevents inflating the price of the share too much.
    #[init]
    pub fn new(
        owner_id: AccountId,
    ) -> Self {
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
            owner_id,
            last_epoch_height: get_epoch_height(),
            last_total_balance: 10 * ONE_NEAR,
            total_share_amount: 10 * ONE_NEAR,
            total_staked_near_amount: 10 * ONE_NEAR,
            accounts: UnorderedMap::new(b"a".to_vec()),
            paused: false,
            account_storage_usage: 0,
            validator_pool: ValidatorPool::new(),
            epoch_requested_stake_amount: 10 * ONE_NEAR,
            epoch_requested_unstake_amount: 0,
            liquidity_pool: LiquidityPool::new(10000 * ONE_NEAR, 300, 30, 7000),
            beneficiaries: UnorderedMap::new(b"b".to_vec()),
        };
        this.measure_account_storage_usage();
        // Staking with the current pool to make sure the staking key is valid.
        this.internal_restake();
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


/// -- Staking pool change methods

#[near_bindgen]
impl LiquidStakingContract {
    /// Distributes rewards and restakes if needed.
    pub fn ping(&mut self) {
        // panic!("ping is not available for liquid staking");
        return
    }

    /// Deposits the attached amount into the inner account of the predecessor.
    #[payable]
    pub fn deposit(&mut self) {
        let need_to_restake = self.internal_ping();

        self.internal_deposit();

        if need_to_restake {
            self.internal_restake();
        }
    }

    /// Deposits the attached amount into the inner account of the predecessor and stakes it.
    #[payable]
    pub fn deposit_and_stake(&mut self) {
        self.internal_ping();

        let amount = self.internal_deposit();
        self.internal_stake(amount);

        self.internal_restake();
    }

    /// Withdraws the entire unstaked balance from the predecessor account.
    /// It's only allowed if the `unstake` action was not performed in the four most recent epochs.
    pub fn withdraw_all(&mut self) {
        let need_to_restake = self.internal_ping();

        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id);
        self.internal_withdraw(account.unstaked);

        if need_to_restake {
            self.internal_restake();
        }
    }

    /// Withdraws the non staked balance for given account.
    /// It's only allowed if the `unstake` action was not performed in the four most recent epochs.
    pub fn withdraw(&mut self, amount: U128) {
        let need_to_restake = self.internal_ping();

        let amount: Balance = amount.into();
        self.internal_withdraw(amount);

        if need_to_restake {
            self.internal_restake();
        }
    }

    /// Stakes all available unstaked balance from the inner account of the predecessor.
    pub fn stake_all(&mut self) {
        // Stake action always restakes
        self.internal_ping();

        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id);
        self.internal_stake(account.unstaked);

        self.internal_restake();
    }

    /// Stakes the given amount from the inner account of the predecessor.
    /// The inner account should have enough unstaked balance.
    pub fn stake(&mut self, amount: U128) {
        // Stake action always restakes
        self.internal_ping();

        let amount: Balance = amount.into();
        self.internal_stake(amount);

        self.internal_restake();
    }

    /// Unstakes all staked balance from the inner account of the predecessor.
    /// The new total unstaked balance will be available for withdrawal in four epochs.
    pub fn unstake_all(&mut self) {
        // Unstake action always restakes
        self.internal_ping();

        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id);
        let amount = self.staked_amount_from_num_shares_rounded_down(account.stake_shares);
        self.inner_unstake(amount);

        self.internal_restake();
    }

    /// Unstakes the given amount from the inner account of the predecessor.
    /// The inner account should have enough staked balance.
    /// The new total unstaked balance will be available for withdrawal in four epochs.
    pub fn unstake(&mut self, amount: U128) {
        // Unstake action always restakes
        self.internal_ping();

        let amount: Balance = amount.into();
        self.inner_unstake(amount);

        self.internal_restake();
    }
}

/// -- Staking pool view methods

#[near_bindgen]
impl LiquidStakingContract {
    /// Returns the unstaked balance of the given account.
    pub fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128 {
        self.get_account(account_id).unstaked_balance
    }

    /// Returns the staked balance of the given account.
    /// NOTE: This is computed from the amount of "stake" shares the given account has and the
    /// current amount of total staked balance and total stake shares on the account.
    pub fn get_account_staked_balance(&self, account_id: AccountId) -> U128 {
        self.get_account(account_id).staked_balance
    }

    /// Returns the total balance of the given account (including staked and unstaked balances).
    pub fn get_account_total_balance(&self, account_id: AccountId) -> U128 {
        let account = self.get_account(account_id);
        (account.unstaked_balance.0 + account.staked_balance.0).into()
    }

    /// Returns `true` if the given account can withdraw tokens in the current epoch.
    pub fn is_account_unstaked_balance_available(&self, account_id: AccountId) -> bool {
        self.get_account(account_id).can_withdraw
    }

    /// Returns the total staking balance.
    pub fn get_total_staked_balance(&self) -> U128 {
        self.total_staked_near_amount.into()
    }

    /// Returns account ID of the staking pool owner.
    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    /// Returns the staking public key
    pub fn get_staking_key(&self) -> PublicKey {
        panic!("no need to specify public key for liquid staking pool");
    }

    /// Returns true if the staking is paused
    pub fn is_staking_paused(&self) -> bool {
        self.paused
    }

    /// Returns human readable representation of the account for the given account ID.
    pub fn get_account(&self, account_id: AccountId) -> HumanReadableAccount {
        let account = self.internal_get_account(&account_id);
        HumanReadableAccount {
            account_id: account_id.clone(),
            unstaked_balance: account.unstaked.into(),
            staked_balance: self
                .staked_amount_from_num_shares_rounded_down(account.stake_shares)
                .into(),
            unstaked_available_epoch_height: account.unstaked_available_epoch_height,
            can_withdraw: account.unstaked_available_epoch_height <= get_epoch_height(),
            liquidity_pool_share: self.liquidity_pool.get_account_shares(&account_id).into(),
            liquidity_pool_share_value: self.liquidity_pool.get_account_value(&account_id, &self.internal_get_context()).into(),
            liquidity_pool_share_percentage: self.liquidity_pool.get_account_shares_percentage(&account_id),
        }
    }

    /// Returns the number of accounts that have positive balance on this staking pool.
    pub fn get_number_of_accounts(&self) -> u64 {
        self.accounts.len()
    }

    /// Returns the list of accounts
    pub fn get_accounts(&self, from_index: u64, limit: u64) -> Vec<HumanReadableAccount> {
        let keys = self.accounts.keys_as_vector();

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| self.get_account(keys.get(index).unwrap()))
            .collect()
    }

}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env};

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
