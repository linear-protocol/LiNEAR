use near_sdk::{
    Balance, EpochHeight, Gas,
};
use uint::construct_uint;

// -- CONSTANTS

/// No deposit balance.
pub const NO_DEPOSIT: Balance = 0;
/// one NEAR
pub const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000;
/// The number of epochs required for the locked balance to become unlocked.
/// NOTE: The actual number of epochs when the funds are unlocked is 3. But there is a corner case
/// when the unstaking promise can arrive at the next epoch, while the inner state is already
/// updated in the previous epoch. It will not unlock the funds for 4 epochs.
pub const NUM_EPOCHS_TO_UNLOCK: EpochHeight = 4;
/// One hundred percent in 10,000
pub const ONE_HUNDRED_PERCENT: u32 = 10_000;
/// DAO treasuary account should be configurable
pub const TREASURY_ACCOUNT: &str = "treasury";

/// -- GAS

pub const TGAS: u64 = 1_000_000_000_000;

pub const GAS_EPOCH_STAKE: Gas = Gas(75 * TGAS);
pub const GAS_EPOCH_UNSTAKE: Gas = Gas(75 * TGAS);
pub const GAS_EPOCH_UPDATE_REWARDS: Gas = Gas(75 * TGAS);
pub const GAS_EPOCH_WITHDRAW: Gas = Gas(75 * TGAS);

pub const GAS_EXT_DEPOSIT_AND_STAKE: Gas = Gas(75 * TGAS);
pub const GAS_EXT_UNSTAKE: Gas = Gas(75 * TGAS);
pub const GAS_EXT_GET_BALANCE: Gas = Gas(25 * TGAS);
pub const GAS_EXT_WITHDRAW: Gas = Gas(75 * TGAS);

pub const GAS_CB_VALIDATOR_STAKED: Gas = Gas(25 * TGAS);
pub const GAS_CB_VALIDATOR_UNSTAKED: Gas = Gas(25 * TGAS);
pub const GAS_CB_VALIDATOR_GET_BALANCE: Gas = Gas(25 * TGAS);
pub const GAS_CB_VALIDATOR_WITHDRAW: Gas = Gas(25 * TGAS);

// -- COMMON TYPES

/// Type for stake share (LiNEAR) balance
pub type ShareBalance = u128;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}
