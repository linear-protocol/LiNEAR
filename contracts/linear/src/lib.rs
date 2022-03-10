use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize,},
    serde::{Deserialize, Serialize,},
    json_types::{U128},
    collections::{UnorderedMap, Vector, UnorderedSet},
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
mod stake;
mod validator_pool;
mod epoch_actions;
mod fungible_token;
mod liquidity_pool;
mod farm;

use crate::types::*;
use crate::utils::*;
use crate::errors::*;
use crate::account::*;
use crate::validator_pool::*;
use crate::farm::{Farm};
pub use crate::fungible_token::*;
pub use crate::liquidity_pool::*;

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
    /// The account ID of the owner who's running the liquid staking contract.
    owner_id: AccountId,
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
    /// Whether the staking is paused.
    /// When paused, the account unstakes everything (stakes 0) and doesn't restake.
    /// It doesn't affect the staking shares or reward distribution.
    /// Pausing is useful for node maintenance. Only the owner can pause and resume staking.
    /// The contract is not paused by default.
    paused: bool,

    /// The storage size in bytes for one account.
    account_storage_usage: StorageUsage,

    /// Beneficiaries for staking rewards.
    beneficiaries: UnorderedMap<AccountId, Fraction>,
  
    /// The single-direction liquidity pool that enables instant unstake
    liquidity_pool: LiquidityPool,
  
    // --- Validator Pool ---

    /// The validator pool that manage the actions against validators
    validator_pool: ValidatorPool,
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

    // --- Staking Farm ---

    /// Farm tokens.
    farms: Vector<Farm>,
    /// Active farms: indicies into `farms`.
    active_farms: Vec<u64>,
    /// Authorized users, allowed to add farms.
    /// This is done to prevent farm spam with random tokens.
    /// Should not be a large number.
    // authorized_users: UnorderedSet<AccountId>,
    /// Authorized tokens for farms.
    /// Required because any contract can call method with ft_transfer_call, so must verify that contract will accept it.
    authorized_farm_tokens: UnorderedSet<AccountId>,
}

#[near_bindgen]
impl LiquidStakingContract {
    /// Initializes the contract with the given owner_id.
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
            owner_id: owner_id.clone(),
            treasury_id: owner_id.clone(),
            total_share_amount: 10 * ONE_NEAR,
            total_staked_near_amount: 10 * ONE_NEAR,
            accounts: UnorderedMap::new(StorageKey::Accounts),
            paused: false,
            account_storage_usage: 0,
            beneficiaries: UnorderedMap::new(StorageKey::Beneficiaries),
            liquidity_pool: LiquidityPool::new(LiquidityPoolConfig::default()),
            // Validator Pool
            validator_pool: ValidatorPool::new(),
            epoch_requested_stake_amount: 10 * ONE_NEAR,
            epoch_requested_unstake_amount: 0,
            stake_amount_to_settle: 0,
            unstake_amount_to_settle: 0,
            last_settlement_epoch: 0,
            // Staking Farm
            farms: Vector::new(StorageKey::Farms),
            active_farms: Vec::new(),
            // authorized_users: UnorderedSet::new(StorageKey::AuthorizedUsers),
            authorized_farm_tokens: UnorderedSet::new(StorageKey::AuthorizedFarmTokens),
        };
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
