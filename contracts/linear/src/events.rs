use crate::types::*;
use near_sdk::{
    AccountId, Balance, log,
    serde_json::{json},
};

pub fn log_stake_attempt(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "stake.attempt",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_stake_success(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "stake.success",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_stake_failed(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "stake.failed",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_unstake_attempt(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "unstake.attempt",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_unstake_success(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "unstake.success",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_unstake_failed(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "unstake.failed",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_withdraw_attempt(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "withdraw.attempt",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_withdraw_success(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "withdraw.success",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_withdraw_failed(
    validator_id: &AccountId,
    amount: Balance
) {
    log!(
        json!({
            "event": "withdraw.failed",
            "validator_id": validator_id,
            "amount": amount
        })
        .to_string()
    );
}

pub fn log_new_balance(
    validator_id: &AccountId,
    old_balance: Balance,
    new_balance: Balance,
    rewards: Balance
) {
    log!(
        json!({
            "event": "balance.update",
            "validator_id": validator_id,
            "old_balance": old_balance,
            "new_balance": new_balance,
            "rewards": rewards
        })
        .to_string()
    );
}

pub fn log_linear_minted(
    account_id: &AccountId,
    amount: ShareBalance,
) {
    log!(
        json!({
            "event": "linear.mint",
            "account_id": account_id,
            "amount": amount
        })
        .to_string()
    );
}
