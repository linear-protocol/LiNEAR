use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
    AccountId, Balance, EpochHeight,
};
use std::collections::HashMap;

use crate::types::*;

/// Inner account data of a delegate.
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct Account {
    /// The unstaked balance. It represents the amount the account has on this contract that
    /// can either be staked or withdrawn.
    pub unstaked: Balance,
    /// The amount of "stake" shares. Every stake share corresponds to the amount of staked balance.
    /// NOTE: The number of shares should always be less or equal than the amount of staked balance.
    /// This means the price of stake share should always be at least `1`.
    /// The price of stake share can be computed as `total_staked_balance` / `total_share_amount`.
    pub stake_shares: ShareBalance,
    /// The minimum epoch height when the withdrawn is allowed.
    /// This changes after unstaking action, because the amount is still locked for 3 epochs.
    pub unstaked_available_epoch_height: EpochHeight,
    /// Farmed tokens withdrawn from the farm but not from the contract.
    pub amounts: HashMap<AccountId, Balance>,
    /// Last claimed reward for each active farm.
    pub last_farm_reward_per_share: HashMap<u64, U256>,
}

/// Represents an account structure readable by humans.
/// This struct comes from staking pool contract.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HumanReadableAccount {
    pub account_id: AccountId,
    /// The unstaked balance that can be withdrawn or staked.
    pub unstaked_balance: U128,
    /// The amount balance staked at the current "stake" share price.
    pub staked_balance: U128,
    /// Whether the unstaked balance is available for withdrawal now.
    pub can_withdraw: bool,
}

/// AccountDetailsView contains all fields from HumanReadableAccount plus:
/// - `unstaked_available_epoch_height` for calculating account unstake waiting time
/// - liquidity pool account data
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountDetailsView {
    pub account_id: AccountId,
    /// The unstaked balance that can be withdrawn or staked.
    pub unstaked_balance: U128,
    /// The amount balance staked at the current "stake" share price.
    pub staked_balance: U128,
    /// The minimum epoch height when the withdrawn is allowed.
    /// This changes after unstaking action, because the amount is still locked for 3 epochs.
    pub unstaked_available_epoch_height: EpochHeight,
    /// Whether the unstaked balance is available for withdrawal now.
    pub can_withdraw: bool,
    /// The liquidity pool share of the account
    pub liquidity_pool_share: U128,
    /// The value of the liquidity pool share in $NEAR
    pub liquidity_pool_share_value: U128,
    /// The liquidity pool share ratio of the total supply in basis points
    pub liquidity_pool_share_ratio_in_basis_points: u32,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            unstaked: 0,
            stake_shares: 0,
            unstaked_available_epoch_height: 0,
            amounts: HashMap::new(),
            last_farm_reward_per_share: HashMap::new(),
        }
    }
}
