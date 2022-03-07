use crate::*;
use near_contract_standards::fungible_token::events::FtTransfer;

impl LiquidStakingContract {
    pub fn internal_ft_get_account(&self, account_id: &AccountId) -> Account {
        match self.accounts.get(account_id) {
            Some(account) => account,
            None => {
                env::panic_str(format!("The account {} is not registered", &account_id).as_str())
            }
        }
    }

    pub fn internal_ft_deposit(&mut self, account_id: &AccountId, amount: ShareBalance) {
        let mut account = self.internal_ft_get_account(account_id);
        let balance = account.stake_shares;
        if let Some(new_balance) = balance.checked_add(amount) {
            account.stake_shares = new_balance;
            self.internal_save_account(account_id, &account);
            self.total_share_amount = self
                .total_share_amount
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("Balance overflow");
        }
    }

    pub fn internal_ft_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let mut account = self.internal_ft_get_account(account_id);
        let balance = account.stake_shares;
        if let Some(new_balance) = balance.checked_sub(amount) {
            account.stake_shares = new_balance;
            self.internal_save_account(account_id, &account);
            self.total_share_amount = self
                .total_share_amount
                .checked_sub(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("The account doesn't have enough balance");
        }
    }

    /// Inner method to transfer LINEAR from sender to receiver
    pub(crate) fn internal_ft_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        assert_ne!(
            sender_id, receiver_id,
            "Sender and receiver should be different"
        );
        assert!(amount > 0, "The amount should be a positive number");

        self.internal_ft_withdraw(sender_id, amount);
        self.internal_ft_deposit(receiver_id, amount);

        FtTransfer {
            old_owner_id: sender_id,
            new_owner_id: receiver_id,
            amount: &U128(amount),
            memo: memo.as_deref(),
        }
        .emit();
    }

    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        if self.accounts.insert(account_id, &Account::default()).is_some() {
            env::panic_str("The account is already registered");
        }
    }
}
