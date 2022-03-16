use near_sdk::{
    AccountId, log,
    serde::{Serialize},
    serde_json::{json},
    json_types::U128,
};

pub (crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": "linear-protocol",
        "version": "1.0.0",
        "event": result["event"],
        "data": [result["data"]]
    })
    .to_string();
    log!(format!("EVENT_JSON:{}", event_json));
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    // Epoch Actions
    EpochStakeAttempt { validator_id: AccountId, amount: U128 },
    EpochStakeSuccess { validator_id: AccountId, amount: U128 },
    EpochStakeFailed { validator_id: AccountId, amount: U128 },
    EpochUnstakeAttempt { validator_id: AccountId, amount: U128 },
    EpochUnstakeSuccess { validator_id: AccountId, amount: U128 },
    EpochUnstakeFailed { validator_id: AccountId, amount: U128 },
    EpochWithdrawAttempt { validator_id: AccountId, amount: U128 },
    EpochWithdrawSuccess { validator_id: AccountId, amount: U128 },
    EpochWithdrawFailed { validator_id: AccountId, amount: U128 },
    EpochUpdateRewards {
        validator_id: AccountId,
        old_balance: U128,
        new_balance: U128,
        rewards: U128
    },
    // Staking Pool Interface
    Deposit {
        account_id: AccountId,
        amount: U128,
        current_unstaked_balance: U128,
    },
    Withdraw {
        account_id: AccountId,
        amount: U128,
        current_unstaked_balance: U128,
    },
    Stake {
        account_id: AccountId,
        decreased_amount: U128,
        increased_stake_shares: U128,
        current_unstaked_balance: U128,
        current_stake_shares: U128,
    },
    Unstake {
        account_id: AccountId,
        increase_amount: U128,
        decreased_stake_shares: U128,
        current_unstaked_balance: U128,
        current_stake_shares: U128,
        available_epoch_height: u64
    },
}

impl Event {
    pub fn emit(&self) {
        emit_event(&self);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils, AccountId};

    fn alice() -> AccountId {
        AccountId::new_unchecked("alice".to_string())
    }

    #[test]
    fn epoch_stake_attempt() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochStakeAttempt { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_stake_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_stake_success() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochStakeSuccess { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_stake_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_stake_failed() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochStakeFailed { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_stake_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_attempt() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochUnstakeAttempt { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_unstake_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_success() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochUnstakeSuccess { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_unstake_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_failed() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochUnstakeFailed { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_unstake_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_attempt() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochWithdrawAttempt { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_withdraw_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_success() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochWithdrawSuccess { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_withdraw_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_failed() {
        let validator_id = alice();
        let amount = U128(100);
        Event::EpochWithdrawFailed { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_withdraw_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_update_rewards() {
        let validator_id = alice();
        let old_balance = 100;
        let new_balance = 120;
        Event::EpochUpdateRewards { 
            validator_id,
            old_balance: U128(old_balance),
            new_balance: U128(new_balance),
            rewards: U128(new_balance - old_balance)
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear-protocol","version":"1.0.0","event":"epoch_update_rewards","data":[{"validator_id":"alice","old_balance":"100","new_balance":"120","rewards":"20"}]}"#
        );
    }
}
