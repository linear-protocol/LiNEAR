use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128};
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue, EpochHeight, PublicKey};
use near_sdk::collections::{UnorderedMap};


mod types;
mod account;
mod internal;

use crate::types::*;
use crate::account::*;
use crate::internal::*;


/// The number of epochs required for the locked balance to become unlocked.
/// NOTE: The actual number of epochs when the funds are unlocked is 3. But there is a corner case
/// when the unstaking promise can arrive at the next epoch, while the inner state is already
/// updated in the previous epoch. It will not unlock the funds for 4 epochs.
const NUM_EPOCHS_TO_UNLOCK: EpochHeight = 4;


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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct LiquidStakingContract {
    /// The account ID of the owner who's running the staking validator node.
    /// NOTE: This is different from the current account ID which is used as a validator account.
    /// The owner of the staking pool can change staking public key and adjust reward fees.
    pub owner_id: AccountId,
    /// The public key which is used for staking action. It's the public key of the validator node
    /// that validates on behalf of the pool.
    pub stake_public_key: PublicKey,
    /// The last epoch height when `ping` was called.
    pub last_epoch_height: EpochHeight,
    /// The last total balance of the account (consists of staked and unstaked balances).
    pub last_total_balance: Balance,
    /// The total amount of shares. It should be equal to the total amount of shares across all
    /// accounts.
    pub total_stake_shares: NumStakeShares,
    /// The total staked balance.
    pub total_staked_balance: Balance,
    /// The fraction of the reward that goes to the owner of the staking pool for running the
    /// validator node.
    pub reward_fee_fraction: RewardFeeFraction,
    /// Persistent map from an account ID to the corresponding account.
    pub accounts: UnorderedMap<AccountId, Account>,
    /// Whether the staking is paused.
    /// When paused, the account unstakes everything (stakes 0) and doesn't restake.
    /// It doesn't affect the staking shares or reward distribution.
    /// Pausing is useful for node maintenance. Only the owner can pause and resume staking.
    /// The contract is not paused by default.
    pub paused: bool,
}

#[near_bindgen]
impl LiquidStakingContract {

    /// Initializes the contract with the given owner_id, initial staking public key (with ED25519
    /// curve) and initial reward fee fraction that owner charges for the validation work.
    ///
    /// The entire current balance of this contract will be used to stake. This allows contract to
    /// always maintain staking shares that can't be unstaked or withdrawn.
    /// It prevents inflating the price of the share too much.
    #[init]
    pub fn new(
        owner_id: AccountId,
        stake_public_key: PublicKey,
        reward_fee_fraction: RewardFeeFraction,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        reward_fee_fraction.assert_valid();
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "The owner account ID is invalid"
        );
        let account_balance = env::account_balance();
        let total_staked_balance = account_balance; // - STAKE_SHARE_PRICE_GUARANTEE_FUND;
        assert_eq!(
            env::account_locked_balance(),
            0,
            "The staking pool shouldn't be staking at the initialization"
        );
        let mut this = Self {
            owner_id,
            stake_public_key: stake_public_key.into(),
            last_epoch_height: env::epoch_height(),
            last_total_balance: account_balance,
            total_staked_balance,
            total_stake_shares: NumStakeShares::from(total_staked_balance),
            reward_fee_fraction,
            accounts: UnorderedMap::new(b"u".to_vec()),
            paused: false,
        };
        // Staking with the current pool to make sure the staking key is valid.
        this.internal_restake();
        this
    }

    /*******************************/
    /* Staking Pool change methods */
    /*******************************/

    /// Deposits the attached amount into the inner account of the predecessor.
    #[payable]
    pub fn deposit(&mut self) {
        panic!("please use deposit_and_stake instead");
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

    /*******************************/
    /* Staking Pool view methods */
    /*******************************/

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
        self.total_staked_balance.into()
    }

    /// Returns account ID of the staking pool owner.
    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    /// Returns the current reward fee as a fraction.
    pub fn get_reward_fee_fraction(&self) -> RewardFeeFraction {
        self.reward_fee_fraction.clone()
    }

    /// Returns the staking public key
    pub fn get_staking_key(&self) -> PublicKey {
        self.stake_public_key.clone()
    }

    /// Returns true if the staking is paused
    pub fn is_staking_paused(&self) -> bool {
        self.paused
    }

    /// Returns human readable representation of the account for the given account ID.
    pub fn get_account(&self, account_id: AccountId) -> HumanReadableAccount {
        let account = self.internal_get_account(&account_id);
        HumanReadableAccount {
            account_id,
            unstaked_balance: account.unstaked.into(),
            staked_balance: self
                .staked_amount_from_num_shares_rounded_down(account.stake_shares)
                .into(),
            can_withdraw: account.unstaked_available_epoch_height <= env::epoch_height(),
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
    use near_sdk::MockedBlockchain;
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
