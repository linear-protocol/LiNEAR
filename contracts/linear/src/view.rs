use crate::*;
use near_sdk::{
    near_bindgen,
};
use std::collections::HashMap;

/// Represents an account structure readable by humans.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Summary {
    /// Total amount of LiNEAR that was minted (minus burned).
    pub total_share_amount: U128,
    /// Total amount of NEAR that was staked by users to this contract.
    pub total_staked_near_amount: U128,

    /// LiNEAR price
    pub ft_price: U128,

    /// Target NEAR amount in Liquidity Pool
    pub lp_target_amount: U128,
    /// Current NEAR amount in Liquidity Pool
    pub lp_near_amount: U128,
    /// Current LiNEAR amount in Liquidity Pool
    pub lp_staked_share: U128,
    /// Current instant unstake fee in Liquidity Pool.
    /// For example, fee percentage is `30`, which means `0.3%`
    pub lp_swap_fee_percentage: u32,
    /// Total received unstake fee as LiNEAR in Liquidity Pool
    pub lp_total_fee_shares: U128,

    /// Number of nodes in validator pool
    pub validators_num: u64,
}

/// public view functions
#[near_bindgen]
impl LiquidStakingContract {
    pub fn get_total_share_amount(& self) -> ShareBalance {
        self.total_share_amount
    }

    pub fn get_total_staked_near_amount(& self) -> Balance {
        self.total_staked_near_amount
    }

    pub fn get_beneficiaries(& self) -> HashMap<AccountId, Fraction> {
        self.internal_get_beneficiaries()
    }

    pub fn get_summary(&self) -> Summary {
        Summary {
            total_share_amount: self.total_share_amount.into(),
            total_staked_near_amount: self.total_staked_near_amount.into(),
            ft_price: self.ft_price(),
            lp_target_amount: self.liquidity_pool.expected_near_amount.into(),
            lp_near_amount: self.liquidity_pool.amounts[0].into(),
            lp_staked_share: self.liquidity_pool.amounts[1].into(),
            lp_swap_fee_percentage: self.liquidity_pool.get_current_swap_fee_percentage(10 * ONE_NEAR),
            lp_total_fee_shares: self.liquidity_pool.total_fee_shares.into(),
            validators_num: self.validator_pool.count()
        }
    }
}
