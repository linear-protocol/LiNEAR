use near_sdk::{
    Balance, EpochHeight,
};
use uint::construct_uint;

// -- CONSTANTS

/// No deposit balance.
pub const NO_DEPOSIT: Balance = 0;
/// one NEAR
pub const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000;

pub const NUM_EPOCHS_TO_UNLOCK: EpochHeight = 4;

// -- COMMON TYPES

/// Type for stake share (LiNEAR) balance
pub type ShareBalance = u128;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}
