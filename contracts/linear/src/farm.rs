//! The staking farm feature has been deprecated.
//! Keep the legacy structs for tracking contract states.

use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    AccountId, Balance, Timestamp,
};

use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct RewardDistribution {
    pub undistributed: Balance,
    pub unclaimed: Balance,
    pub reward_per_share: U256,
    pub reward_round: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Farm {
    pub name: String,
    pub token_id: AccountId,
    pub amount: Balance,
    pub start_date: Timestamp,
    pub end_date: Timestamp,
    pub last_distribution: RewardDistribution,
}
