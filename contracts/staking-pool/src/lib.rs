use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128};
use near_sdk::collections::{LookupMap};
use near_sdk::{ env, near_bindgen, AccountId, PanicOnDefault, Promise };

/// Staking pool interface
trait StakingPool {
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

/// mockup of staking pool, for testing
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MockStakingPool {
    deposits: LookupMap<AccountId, u128>,
    staked: LookupMap<AccountId, u128>
}

#[near_bindgen]
impl MockStakingPool {
    #[init]
    pub fn new() -> Self {
        Self {
            deposits: LookupMap::new(b"d"),
            staked: LookupMap::new(b"s")
        }
    }
}

#[near_bindgen]
impl StakingPool for MockStakingPool {
    fn get_account_staked_balance(&self, account_id: AccountId) -> U128 {
        U128::from(self.internal_get_staked(&account_id))
    }

    fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128 {
        U128::from(self.internal_get_unstaked_deposit(&account_id))
    }

    fn get_account_total_balance(&self, account_id: AccountId) -> U128 {
        U128::from(
            self.internal_get_unstaked_deposit(&account_id) + self.internal_get_staked(&account_id)
        )
    }

    #[payable]
    fn deposit(&mut self) {
        self.internal_deposit()
    }

    #[payable]
    fn deposit_and_stake(&mut self) {
        let account_id = env::predecessor_account_id();

        self.internal_deposit();

        let amount = self.internal_get_unstaked_deposit(&account_id);
        self.internal_stake(amount);
    }

    fn withdraw(&mut self, amount: U128) {
        let account_id = env::predecessor_account_id();
        self.internal_withdraw(&account_id, amount.0);
    }

    fn withdraw_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let unstaked = self.internal_get_unstaked_deposit(&account_id);
        self.internal_withdraw(&account_id, unstaked);
    }

    fn stake(&mut self, amount: U128) {
        self.internal_stake(amount.0)
    }

    fn unstake(&mut self, amount: U128) {
        self.internal_unstake(amount.0);
    }

    fn unstake_all(&mut self) {
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
        let staked_amount = self.internal_get_staked(&account_id);
        assert!(staked_amount > 0);

        let new_amount = staked_amount + amount.0;
        self.staked.insert(&account_id, &new_amount);
    }
}

impl MockStakingPool {
    fn internal_deposit(&mut self) {
        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();
        assert!(amount > 0);

        let current_deposit = self.internal_get_unstaked_deposit(&account_id);
        let new_deposit = current_deposit + amount;

        self.deposits.insert(&account_id, &new_deposit);
    }
    
    fn internal_stake(&mut self, amount: u128) {
        let account_id = env::predecessor_account_id();
        let unstaked_deposit = self.internal_get_unstaked_deposit(&account_id);
        assert!(unstaked_deposit >= amount);

        let new_deposit = unstaked_deposit - amount;
        let new_staked = self.internal_get_staked(&account_id) + amount;

        self.deposits.insert(&account_id, &new_deposit);
        self.staked.insert(&account_id, &new_staked);
    }

    fn internal_unstake(&mut self, amount: u128) {
        let account_id = env::predecessor_account_id();
        let staked = self.internal_get_staked(&account_id);
        assert!(staked >= amount);

        let unstaked_deposit = self.internal_get_unstaked_deposit(&account_id);
        let new_deposit  = unstaked_deposit + amount;
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

    fn internal_get_unstaked_deposit(& self, account_id: &AccountId) -> u128 {
        self.deposits.get(account_id).unwrap_or_default()
    }

    fn internal_get_staked(& self, account_id: &AccountId) -> u128 {
        self.staked.get(&account_id).unwrap_or_default()
    }
}
