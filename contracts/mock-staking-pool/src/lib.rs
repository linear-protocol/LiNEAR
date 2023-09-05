use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, PanicOnDefault, Promise};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HumanReadableAccount {
    pub account_id: AccountId,
    /// The unstaked balance that can be withdrawn or staked.
    pub unstaked_balance: U128,
    /// The amount balance staked at the current "stake" share price.
    pub staked_balance: U128,
    /// Whether the unstaked balance is available for withdrawal now.
    pub can_withdraw: bool,
}

/// Staking pool interface
trait StakingPool {
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

/// mockup of staking pool, for testing
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MockStakingPool {
    deposits: LookupMap<AccountId, u128>,
    staked: LookupMap<AccountId, u128>,
    /// for testing purpose, simulates contract panic
    panic: bool,
    get_account_fail: bool,

    staked_delta: u128,
    unstaked_delta: u128,
}

#[near_bindgen]
impl MockStakingPool {
    #[init]
    pub fn new() -> Self {
        Self {
            deposits: LookupMap::new(b"d"),
            staked: LookupMap::new(b"s"),
            panic: false,
            get_account_fail: false,
            staked_delta: 0,
            unstaked_delta: 0,
        }
    }
}

#[near_bindgen]
impl StakingPool for MockStakingPool {
    fn get_account_staked_balance(&self, account_id: AccountId) -> U128 {
        require!(!self.panic, "Test Panic!");
        U128::from(self.internal_get_staked(&account_id))
    }

    fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128 {
        require!(!self.panic, "Test Panic!");
        U128::from(self.internal_get_unstaked_deposit(&account_id))
    }

    fn get_account_total_balance(&self, account_id: AccountId) -> U128 {
        require!(!self.panic, "Test Panic!");
        U128::from(
            self.internal_get_unstaked_deposit(&account_id) + self.internal_get_staked(&account_id),
        )
    }

    fn get_account(&self, account_id: AccountId) -> HumanReadableAccount {
        require!(!self.panic, "Test Panic!");
        require!(
            !self.get_account_fail,
            "get_account() failed, for testing purpose",
        );
        HumanReadableAccount {
            account_id: account_id.clone(),
            staked_balance: U128::from(self.internal_get_staked(&account_id)),
            unstaked_balance: U128::from(self.internal_get_unstaked_deposit(&account_id)),
            can_withdraw: true,
        }
    }

    #[payable]
    fn deposit(&mut self) {
        require!(!self.panic, "Test Panic!");
        self.internal_deposit();
    }

    #[payable]
    fn deposit_and_stake(&mut self) {
        require!(!self.panic, "Test Panic!");
        let amount = self.internal_deposit();
        self.internal_stake(amount);
    }

    fn withdraw(&mut self, amount: U128) {
        require!(!self.panic, "Test Panic!");
        let account_id = env::predecessor_account_id();
        self.internal_withdraw(&account_id, amount.0);
    }

    fn withdraw_all(&mut self) {
        require!(!self.panic, "Test Panic!");
        let account_id = env::predecessor_account_id();
        let unstaked = self.internal_get_unstaked_deposit(&account_id);
        self.internal_withdraw(&account_id, unstaked);
    }

    fn stake(&mut self, amount: U128) {
        require!(!self.panic, "Test Panic!");
        self.internal_stake(amount.0)
    }

    fn unstake(&mut self, amount: U128) {
        require!(!self.panic, "Test Panic!");
        self.internal_unstake(amount.0);
    }

    fn unstake_all(&mut self) {
        require!(!self.panic, "Test Panic!");
        let account_id = env::predecessor_account_id();
        let staked_amount = self.internal_get_staked(&account_id);
        self.internal_unstake(staked_amount);
    }
}

#[near_bindgen]
impl MockStakingPool {
    /// manually generate some reward for the caller,
    /// for testing purpose only
    pub fn add_reward(&mut self, amount: U128) {
        let account_id = env::predecessor_account_id();
        self.add_reward_for(amount, account_id);
    }

    pub fn add_reward_for(&mut self, amount: U128, account_id: AccountId) {
        let staked_amount = self.internal_get_staked(&account_id);
        assert!(staked_amount > 0);

        let new_amount = staked_amount + amount.0;
        self.staked.insert(&account_id, &new_amount);
    }

    pub fn set_panic(&mut self, panic: bool) {
        self.panic = panic;
    }

    pub fn set_get_account_fail(&mut self, value: bool) {
        self.get_account_fail = value;
    }

    pub fn set_balance_delta(&mut self, staked_delta: U128, unstaked_delta: U128) {
        self.staked_delta = staked_delta.0;
        self.unstaked_delta = unstaked_delta.0;
    }
}

impl MockStakingPool {
    fn internal_deposit(&mut self) -> u128 {
        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();
        assert!(amount > 0);

        let current_deposit = self.internal_get_unstaked_deposit(&account_id);
        let new_deposit = current_deposit + amount;

        self.deposits.insert(&account_id, &new_deposit);
        amount
    }

    fn internal_stake(&mut self, amount: u128) {
        let account_id = env::predecessor_account_id();
        let unstaked_deposit = self.internal_get_unstaked_deposit(&account_id);
        assert!(unstaked_deposit >= amount);

        let new_deposit = unstaked_deposit - amount;
        let new_staked = self.internal_get_staked(&account_id) + amount - self.staked_delta;

        self.deposits.insert(&account_id, &new_deposit);
        self.staked.insert(&account_id, &new_staked);
    }

    fn internal_unstake(&mut self, amount: u128) {
        let account_id = env::predecessor_account_id();
        let staked = self.internal_get_staked(&account_id);
        assert!(staked >= amount);

        let unstaked_deposit = self.internal_get_unstaked_deposit(&account_id);
        let new_deposit = unstaked_deposit + amount + self.unstaked_delta;
        let new_staked = staked - amount;

        self.deposits.insert(&account_id, &new_deposit);
        self.staked.insert(&account_id, &new_staked);
    }

    fn internal_withdraw(&mut self, account_id: &AccountId, amount: u128) {
        let unstaked_amount = self.internal_get_unstaked_deposit(account_id);
        assert!(unstaked_amount >= amount);

        let new_unstaked = unstaked_amount - amount;
        self.deposits.insert(account_id, &new_unstaked);

        Promise::new(account_id.clone()).transfer(amount);
    }

    fn internal_get_unstaked_deposit(&self, account_id: &AccountId) -> u128 {
        self.deposits.get(account_id).unwrap_or_default()
    }

    fn internal_get_staked(&self, account_id: &AccountId) -> u128 {
        self.staked.get(account_id).unwrap_or_default()
    }
}
