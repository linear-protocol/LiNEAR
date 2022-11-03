use crate::*;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::events::FtTransfer;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, env, near_bindgen, AccountId, Balance, Gas, PromiseOrValue, PromiseResult,
};

// allocate enough gas for ft_resolve_transfer() to avoid unexpected failure
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(12 * TGAS);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(35 * TGAS + GAS_FOR_RESOLVE_TRANSFER.0);

#[ext_contract(ext_fungible_token_receiver)]
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft_self)]
trait FungibleTokenResolver {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}

#[near_bindgen]
impl FungibleTokenCore for LiquidStakingContract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let amount = amount.into();
        self.internal_ft_transfer(&sender_id, &receiver_id, amount, memo);
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        // Ensure minimum required gas is attached
        require!(
            env::prepaid_gas() > GAS_FOR_FT_TRANSFER_CALL,
            format!(
                "{}. require at least {:?}",
                ERR_NO_ENOUGH_GAS, GAS_FOR_FT_TRANSFER_CALL
            )
        );
        let sender_id = env::predecessor_account_id();
        let amount = amount.into();
        self.internal_ft_transfer(&sender_id, &receiver_id, amount, memo);
        // Initiating receiver's call and the callback
        ext_fungible_token_receiver::ft_on_transfer(
            sender_id.clone(),
            amount.into(),
            msg,
            receiver_id.clone(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_ft_self::ft_resolve_transfer(
            sender_id,
            receiver_id,
            amount.into(),
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_share_amount.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        let account = self.internal_get_account(&account_id);
        account.stake_shares.into()
    }
}

#[near_bindgen]
impl FungibleTokenResolver for LiquidStakingContract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        if unused_amount > 0 {
            let mut receiver = self.internal_get_account(&receiver_id);
            let receiver_balance = receiver.stake_shares;
            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);
                receiver.stake_shares -= refund_amount;
                self.internal_save_account(&receiver_id, &receiver);

                let mut sender = self.internal_get_account(&sender_id);
                sender.stake_shares += refund_amount;
                self.internal_save_account(&sender_id, &sender);

                FtTransfer {
                    old_owner_id: &receiver_id,
                    new_owner_id: &sender_id,
                    amount: &U128(refund_amount),
                    memo: Some("refund"),
                }
                .emit();

                return (amount - refund_amount).into();
            }
        }
        amount.into()
    }
}

impl LiquidStakingContract {
    pub(crate) fn internal_ft_get_account(&self, account_id: &AccountId) -> Account {
        match self.accounts.get(account_id) {
            Some(account) => account,
            None => {
                env::panic_str(format!("The account {} is not registered", &account_id).as_str())
            }
        }
    }

    pub(crate) fn internal_ft_deposit(&mut self, account_id: &AccountId, amount: ShareBalance) {
        self.assert_running();

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

    pub(crate) fn internal_ft_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        self.assert_running();

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
        require!(
            sender_id != receiver_id,
            "Sender and receiver should be different"
        );
        require!(amount > 0, "The amount should be a positive number");

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
}
