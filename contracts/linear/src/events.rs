use near_sdk::{
    AccountId, log,
    serde::{Serialize},
    serde_json::{json},
    json_types::U128,
};

const EVENT_STANDARD: &str = "linear";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    // Epoch Actions
    EpochStakeAttempt { validator_id: &'a AccountId, amount: &'a U128 },
    EpochStakeSuccess { validator_id: &'a AccountId, amount: &'a U128 },
    EpochStakeFailed { validator_id: &'a AccountId, amount: &'a U128 },
    EpochUnstakeAttempt { validator_id: &'a AccountId, amount: &'a U128 },
    EpochUnstakeSuccess { validator_id: &'a AccountId, amount: &'a U128 },
    EpochUnstakeFailed { validator_id: &'a AccountId, amount: &'a U128 },
    EpochWithdrawAttempt { validator_id: &'a AccountId, amount: &'a U128 },
    EpochWithdrawSuccess { validator_id: &'a AccountId, amount: &'a U128 },
    EpochWithdrawFailed { validator_id: &'a AccountId, amount: &'a U128 },
    EpochUpdateRewards {
        validator_id: &'a AccountId,
        old_balance: &'a U128,
        new_balance: &'a U128,
        rewards: &'a U128
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
        unstaked_available_epoch_height: u64
    },
    // Liquidity Pool
    InstantUnstake {
        account_id: &'a AccountId,
        unstaked_amount: &'a U128,
        swapped_stake_shares: &'a U128,
        new_unstaked_balance: &'a U128,
        new_stake_shares: &'a U128,
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
        stake_shares_in: &'a U128,
        requested_amount: &'a U128,
        received_amount: &'a U128,
        swap_fee_amount: &'a U128,
        swap_fee_stake_shares: &'a U128,
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
pub (crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": EVENT_STANDARD,
        "version": "1.0.0",
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
        Event::EpochStakeAttempt { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_stake_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_stake_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochStakeSuccess { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_stake_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_stake_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochStakeFailed { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_stake_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_attempt() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochUnstakeAttempt { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_unstake_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochUnstakeSuccess { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_unstake_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_unstake_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochUnstakeFailed { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_unstake_failed","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_attempt() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochWithdrawAttempt { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_withdraw_attempt","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_success() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochWithdrawSuccess { validator_id, amount }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"epoch_withdraw_success","data":[{"validator_id":"alice","amount":"100"}]}"#
        );
    }

    #[test]
    fn epoch_withdraw_failed() {
        let validator_id = &alice();
        let amount = &U128(100);
        Event::EpochWithdrawFailed { validator_id, amount }.emit();
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
        let unstaked_amount = &U128(97);
        let swapped_stake_shares = &U128(100);
        let new_unstaked_balance = &U128(111);
        let new_stake_shares = &U128(99);
        Event::InstantUnstake {
            account_id,
            unstaked_amount,
            swapped_stake_shares,
            new_unstaked_balance,
            new_stake_shares,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"instant_unstake","data":[{"account_id":"alice","unstaked_amount":"97","swapped_stake_shares":"100","new_unstaked_balance":"111","new_stake_shares":"99"}]}"#
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
            minted_shares
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
            received_linear
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
        let stake_shares_in = &U128(100);
        let requested_amount = &U128(100);
        let received_amount = &U128(97);
        let swap_fee_amount = &U128(3);
        let swap_fee_stake_shares = &U128(3);
        let treasury_fee_stake_shares = &U128(1);
        let pool_fee_stake_shares = &U128(2);
        let total_fee_shares = &U128(1022);
        Event::LiquidityPoolSwapFee {
            stake_shares_in,
            requested_amount,
            received_amount,
            swap_fee_amount,
            swap_fee_stake_shares,
            treasury_fee_stake_shares,
            pool_fee_stake_shares,
            total_fee_shares,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"linear","version":"1.0.0","event":"liquidity_pool_swap_fee","data":[{"stake_shares_in":"100","requested_amount":"100","received_amount":"97","swap_fee_amount":"3","swap_fee_stake_shares":"3","treasury_fee_stake_shares":"1","pool_fee_stake_shares":"2","total_fee_shares":"1022"}]}"#
        );
    }
}
