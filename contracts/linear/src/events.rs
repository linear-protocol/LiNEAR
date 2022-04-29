use near_sdk::{json_types::U128, log, serde::Serialize, serde_json::json, AccountId};

const EVENT_STANDARD: &str = "linear";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    // Epoch Actions
    EpochStakeAttempt {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochStakeSuccess {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochStakeFailed {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochUnstakeAttempt {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochUnstakeSuccess {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochUnstakeFailed {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochWithdrawAttempt {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochWithdrawSuccess {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochWithdrawFailed {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    EpochUpdateRewards {
        validator_id: &'a AccountId,
        old_balance: &'a U128,
        new_balance: &'a U128,
        rewards: &'a U128,
    },
    EpochCleanup {
        stake_amount_to_settle: &'a U128,
        unstake_amount_to_settle: &'a U128,
    },
    // Drain Operations
    DrainUnstakeAttempt {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    DrainUnstakeSuccess {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    DrainUnstakeFailed {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    DrainWithdrawAttempt {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    DrainWithdrawSuccess {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    DrainWithdrawFailed {
        validator_id: &'a AccountId,
        amount: &'a U128,
    },
    // Sync validator balance
    BalanceSyncedFromValidator {
        validator_id: &'a AccountId,
        staked_balance: &'a U128,
        unstaked_balance: &'a U128,
    },
    // Staking Pool Interface
    Deposit {
        account_id: &'a AccountId,
        amount: &'a U128,
        new_unstaked_balance: &'a U128,
    },
    Withdraw {
        account_id: &'a AccountId,
        amount: &'a U128,
        new_unstaked_balance: &'a U128,
    },
    Stake {
        account_id: &'a AccountId,
        staked_amount: &'a U128,
        minted_stake_shares: &'a U128,
        new_unstaked_balance: &'a U128,
        new_stake_shares: &'a U128,
    },
    Unstake {
        account_id: &'a AccountId,
        unstaked_amount: &'a U128,
        burnt_stake_shares: &'a U128,
        new_unstaked_balance: &'a U128,
        new_stake_shares: &'a U128,
        unstaked_available_epoch_height: u64,
    },
    // Validators
    ValidatorAdded {
        account_id: &'a AccountId,
        weight: u16
    },
    ValidatorUpdated {
        account_id: &'a AccountId,
        weight: u16
    },
    ValidatorRemoved {
        account_id: &'a AccountId,
    },
    // Liquidity Pool
    InstantUnstake {
        account_id: &'a AccountId,
        /// The actually received NEAR excluding fees
        unstaked_amount: &'a U128,
        /// The swapped-in stake shares
        swapped_stake_shares: &'a U128,
        new_unstaked_balance: &'a U128,
        new_stake_shares: &'a U128,
        /// The fee of instant unstake in NEAR
        fee_amount: &'a U128,
    },
    AddLiquidity {
        account_id: &'a AccountId,
        amount: &'a U128,
        minted_shares: &'a U128,
    },
    RemoveLiquidity {
        account_id: &'a AccountId,
        burnt_shares: &'a U128,
        received_near: &'a U128,
        received_linear: &'a U128,
    },
    RebalanceLiquidity {
        account_id: &'a AccountId,
        increased_amount: &'a U128,
        burnt_stake_shares: &'a U128,
    },
    LiquidityPoolSwapFee {
        account_id: &'a AccountId,
        stake_shares_in: &'a U128,
        requested_amount: &'a U128,
        received_amount: &'a U128,
        fee_amount: &'a U128,
        fee_stake_shares: &'a U128,
        treasury_fee_stake_shares: &'a U128,
        pool_fee_stake_shares: &'a U128,
        total_fee_shares: &'a U128,
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        emit_event(&self);
    }
}

// Emit event that follows NEP-297 standard: https://nomicon.io/Standards/EventsFormat
// Arguments
// * `standard`: name of standard, e.g. nep171
// * `version`: e.g. 1.0.0
// * `event`: type of the event, e.g. nft_mint
// * `data`: associate event data. Strictly typed for each set {standard, version, event} inside corresponding NEP
pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": EVENT_STANDARD,
        "version": EVENT_STANDARD_VERSION,
        "event": result["event"],
        "data": [result["data"]]
    })
    .to_string();
    log!(format!("EVENT_JSON:{}", event_json));
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
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochStakeAttempt {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_stake_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_stake_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochStakeSuccess {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_stake_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_stake_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochStakeFailed {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_stake_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_attempt() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochUnstakeAttempt {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_unstake_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochUnstakeSuccess {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_unstake_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochUnstakeFailed {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_unstake_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_cleanup() {
        let stake_amount_to_settle = &U128(100);
        let unstake_amount_to_settle = &U128(0);
        Event::EpochCleanup {
            stake_amount_to_settle,
            unstake_amount_to_settle,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_cleanup","data":[{"stake_amount_to_settle":"100","unstake_amount_to_settle":"0"}]}"#
        );
    }

    #[test]
    fn drain_unstake_attempt() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::DrainUnstakeAttempt {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"drain_unstake_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn drain_unstake_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::DrainUnstakeSuccess {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"drain_unstake_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn drain_unstake_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::DrainUnstakeFailed {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"drain_unstake_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn drain_withdraw_attempt() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::DrainWithdrawAttempt {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"drain_withdraw_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn drain_withdraw_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::DrainWithdrawSuccess {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"drain_withdraw_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn drain_withdraw_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::DrainWithdrawFailed {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"drain_withdraw_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_attempt() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochWithdrawAttempt {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_withdraw_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochWithdrawSuccess {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_withdraw_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochWithdrawFailed {
            validator_id,
            amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_withdraw_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_update_rewards() {
        let validator_id = &alice();
        let old_balance = 100;
        let new_balance = 120;
        Event::EpochUpdateRewards {
            validator_id,
            old_balance: &U128(old_balance),
            new_balance: &U128(new_balance),
            rewards: &U128(new_balance - old_balance),
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_update_rewards","data":[{"validator_id":"alice","old_balance":"100","new_balance":"120","rewards":"20"}]}"#
        );
    }

    #[test]
    fn balance_synced_from_validator() {
        let validator_id = &alice();
        let staked_balance = &U128(300);
        let unstaked_balance = &U128(200);
        Event::BalanceSyncedFromValidator {
            validator_id,
            staked_balance,
            unstaked_balance,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"balance_synced_from_validator","data":[{"validator_id":"alice","staked_balance":"300","unstaked_balance":"200"}]}"#
        );
    }

    #[test]
    fn deposit() {
        let account_id = &alice();
        let amount = &U128(100);
        let new_unstaked_balance = &U128(200);
        Event::Deposit {
            account_id,
            amount,
            new_unstaked_balance,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"deposit","data":[{"account_id":"alice","amount":"100","new_unstaked_balance":"200"}]}"#
        );
    }

    #[test]
    fn withdraw() {
        let account_id = &alice();
        let amount = &U128(100);
        let new_unstaked_balance = &U128(50);
        Event::Withdraw {
            account_id,
            amount,
            new_unstaked_balance,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"withdraw","data":[{"account_id":"alice","amount":"100","new_unstaked_balance":"50"}]}"#
        );
    }

    #[test]
    fn stake() {
        let account_id = &alice();
        let staked_amount = &U128(100);
        let minted_stake_shares = &U128(99);
        let new_unstaked_balance = &U128(10);
        let new_stake_shares = &U128(199);
        Event::Stake {
            account_id,
            staked_amount,
            minted_stake_shares,
            new_unstaked_balance,
            new_stake_shares,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"stake","data":[{"account_id":"alice","staked_amount":"100","minted_stake_shares":"99","new_unstaked_balance":"10","new_stake_shares":"199"}]}"#
        );
    }

    #[test]
    fn unstake() {
        let account_id = &alice();
        let unstaked_amount = &U128(101);
        let burnt_stake_shares = &U128(100);
        let new_unstaked_balance = &U128(111);
        let new_stake_shares = &U128(99);
        let unstaked_available_epoch_height = 932;
        Event::Unstake {
            account_id,
            unstaked_amount,
            burnt_stake_shares,
            new_unstaked_balance,
            new_stake_shares,
            unstaked_available_epoch_height,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"unstake","data":[{"account_id":"alice","unstaked_amount":"101","burnt_stake_shares":"100","new_unstaked_balance":"111","new_stake_shares":"99","unstaked_available_epoch_height":932}]}"#
        );
    }

    #[test]
    fn instant_unstake() {
        let account_id = &alice();
        let unstaked_amount = &U128(98);
        let swapped_stake_shares = &U128(100);
        let new_unstaked_balance = &U128(111);
        let new_stake_shares = &U128(99);
        let fee_amount = &U128(3);
        Event::InstantUnstake {
            account_id,
            unstaked_amount,
            swapped_stake_shares,
            new_unstaked_balance,
            new_stake_shares,
            fee_amount,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"instant_unstake","data":[{"account_id":"alice","unstaked_amount":"98","swapped_stake_shares":"100","new_unstaked_balance":"111","new_stake_shares":"99","fee_amount":"3"}]}"#
        );
    }

    #[test]
    fn validator_added() {
        let account_id = &alice();
        let weight: u16 = 10;
        Event::ValidatorAdded {
            account_id,
            weight,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"validator_added","data":[{"account_id":"alice","weight":10}]}"#
        );
    }

    #[test]
    fn validator_updated() {
        let account_id = &alice();
        let weight: u16 = 10;
        Event::ValidatorUpdated{
            account_id,
            weight,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"validator_updated","data":[{"account_id":"alice","weight":10}]}"#
        );
    }

    #[test]
    fn validator_removed() {
        let account_id = &alice();
        Event::ValidatorRemoved {
            account_id,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"validator_removed","data":[{"account_id":"alice"}]}"#
        );
    }

    #[test]
    fn add_liquidity() {
        let account_id = &alice();
        let amount = &U128(100);
        let minted_shares = &U128(98);
        Event::AddLiquidity {
            account_id,
            amount,
            minted_shares,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"add_liquidity","data":[{"account_id":"alice","amount":"100","minted_shares":"98"}]}"#
        );
    }

    #[test]
    fn remove_liquidity() {
        let account_id = &alice();
        let burnt_shares = &U128(98);
        let received_near = &U128(90);
        let received_linear = &U128(9);
        Event::RemoveLiquidity {
            account_id,
            burnt_shares,
            received_near,
            received_linear,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"remove_liquidity","data":[{"account_id":"alice","burnt_shares":"98","received_near":"90","received_linear":"9"}]}"#
        );
    }

    #[test]
    fn rebalance_liquidity() {
        let account_id = &alice();
        let increased_amount = &U128(100);
        let burnt_stake_shares = &U128(99);
        Event::RebalanceLiquidity {
            account_id,
            increased_amount,
            burnt_stake_shares,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"rebalance_liquidity","data":[{"account_id":"alice","increased_amount":"100","burnt_stake_shares":"99"}]}"#
        );
    }

    #[test]
    fn liquidity_pool_swap_fee() {
        let account_id = &alice();
        let stake_shares_in = &U128(100);
        let requested_amount = &U128(100);
        let received_amount = &U128(97);
        let fee_amount = &U128(3);
        let fee_stake_shares = &U128(3);
        let treasury_fee_stake_shares = &U128(1);
        let pool_fee_stake_shares = &U128(2);
        let total_fee_shares = &U128(1022);
        Event::LiquidityPoolSwapFee {
            account_id,
            stake_shares_in,
            requested_amount,
            received_amount,
            fee_amount,
            fee_stake_shares,
            treasury_fee_stake_shares,
            pool_fee_stake_shares,
            total_fee_shares,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"liquidity_pool_swap_fee","data":[{"account_id":"alice","stake_shares_in":"100","requested_amount":"100","received_amount":"97","fee_amount":"3","fee_stake_shares":"3","treasury_fee_stake_shares":"1","pool_fee_stake_shares":"2","total_fee_shares":"1022"}]}"#
        );
    }
}
