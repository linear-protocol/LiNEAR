use crate::*;

/// -- Staking Pool change methods

#[near_bindgen]
impl LiquidStakingContract {
    /// Please notice ping() is not available for liquid staking.
    /// Keep here for interface consistency.
    pub fn ping(&mut self) {}

    /// Deposits the attached amount into the inner account of the predecessor.
    #[payable]
    pub fn deposit(&mut self) {
        let amount = env::attached_deposit();
        self.internal_deposit(amount);
    }

    /// Deposits the attached amount into the inner account of the predecessor and stakes it.
    #[payable]
    pub fn deposit_and_stake(&mut self) {
        let amount = env::attached_deposit();
        self.internal_deposit(amount);
        self.internal_stake(amount);
    }

    /// Withdraws the entire unstaked balance from the predecessor account.
    /// It's only allowed if the `unstake` action was not performed in the four most recent epochs.
    pub fn withdraw_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id);
        self.internal_withdraw(account.unstaked);
    }

    /// Withdraws the non staked balance for given account.
    /// It's only allowed if the `unstake` action was not performed in the four most recent epochs.
    pub fn withdraw(&mut self, amount: U128) {
        let amount: Balance = amount.into();
        self.internal_withdraw(amount);
    }

    /// Stakes all available unstaked balance from the inner account of the predecessor.
    pub fn stake_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id);
        self.internal_stake(account.unstaked);
    }

    /// Stakes the given amount from the inner account of the predecessor.
    /// The inner account should have enough unstaked balance.
    pub fn stake(&mut self, amount: U128) {
        let amount: Balance = amount.into();
        self.internal_stake(amount);
    }

    /// Unstakes all staked balance from the inner account of the predecessor.
    /// The new total unstaked balance will be available for withdrawal in four epochs.
    pub fn unstake_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id);
        let amount = self.staked_amount_from_num_shares_rounded_down(account.stake_shares);
        self.internal_unstake(amount);
    }

    /// Unstakes the given amount from the inner account of the predecessor.
    /// The inner account should have enough staked balance.
    /// The new total unstaked balance will be available for withdrawal in four epochs.
    pub fn unstake(&mut self, amount: U128) {
        let amount: Balance = amount.into();
        self.internal_unstake(amount);
    }
}

/// -- Base staking change methods
#[near_bindgen]
impl LiquidStakingContract {
    /// Base stake NEAR which will be managed with validators' base stake amount
    pub fn base_deposit_and_stake(&mut self) {
        let amount = env::attached_deposit();
        self.internal_base_deposit(amount);
        self.internal_base_stake(amount);
    }

    pub fn base_withdraw_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let account = self.internal_get_base_account(&account_id);
        self.internal_base_withdraw(account.unstaked);
    }

    pub fn base_withdraw(&mut self, amount: U128) {
        let amount: Balance = amount.into();
        self.internal_base_withdraw(amount);
    }

    pub fn base_unstake_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let account = self.internal_get_base_account(&account_id);
        self.internal_unstake(account.stake_shares);
    }

    pub fn base_unstake(&mut self, amount: U128) {
        let amount: Balance = amount.into();
        self.internal_base_unstake(amount);
    }
}
