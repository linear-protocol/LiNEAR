use crate::*;
use near_sdk::json_types::{U128};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, AccountId, Balance, Gas, 
    PromiseOrValue, PromiseResult,
};
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(8_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

const NO_DEPOSIT: Balance = 0;

#[ext_contract(ext_fungible_token_receiver)]
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_self)]
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
        .then(ext_self::ft_resolve_transfer(
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
        self.total_stake_shares.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        let account = self.internal_get_account(&account_id.into());
        return account.stake_shares.into();
    }
}

#[near_bindgen]
impl FungibleTokenResolver for LiquidStakingContract {
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
                // TODO: conversion between stake_shares (LINEAR) and NEAR amount
                receiver.stake_shares -= refund_amount;
                self.internal_save_account(&receiver_id, &receiver);

                let mut sender = self.internal_get_account(&sender_id);
                // TODO: conversion between stake_shares (LINEAR) and NEAR amount
                sender.stake_shares += refund_amount;
                self.internal_save_account(&sender_id, &sender);

                env::log_str(
                    format!(
                        "Refund {} from {} to {}",
                        refund_amount, receiver_id, sender_id
                    ),
                );
                return (amount - refund_amount).into();
            }
        }
        amount.into()
    }
}
