use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupSet;
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

/// mockup of staking pool, for testing
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MockWhitelist {
    whitelist: LookupSet<AccountId>,
    allow_all: bool,
}

#[near_bindgen]
impl MockWhitelist {
    #[init]
    pub fn new() -> Self {
        Self {
            whitelist: LookupSet::new("w".as_bytes()),
            allow_all: false,
        }
    }

    pub fn add_whitelist(&mut self, account_id: AccountId) {
        self.whitelist.insert(&account_id);
    }

    pub fn allow_all(&mut self) {
        self.allow_all = true;
    }

    pub fn is_whitelisted(&self, staking_pool_account_id: AccountId) -> bool {
        self.allow_all || self.whitelist.contains(&staking_pool_account_id)
    }
}
