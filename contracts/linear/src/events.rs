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
