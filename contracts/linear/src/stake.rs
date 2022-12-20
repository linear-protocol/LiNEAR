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
    /// - (since v1.3.0) Returns the received LiNEAR amount
    #[payable]
    pub fn deposit_and_stake(&mut self) -> U128 {
        let amount = env::attached_deposit();
        self.internal_deposit(amount);
        self.internal_stake(amount).into()
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
    /// - (since v1.3.0) Returns the received LiNEAR amount
    pub fn stake_all(&mut self) -> U128 {
        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id);
        self.internal_stake(account.unstaked).into()
    }

    /// Stakes the given amount from the inner account of the predecessor.
    /// The inner account should have enough unstaked balance.
    /// - (since v1.3.0) Returns the received LiNEAR amount
    pub fn stake(&mut self, amount: U128) -> U128 {
        let amount: Balance = amount.into();
        self.internal_stake(amount).into()
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
