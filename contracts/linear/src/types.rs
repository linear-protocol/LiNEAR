use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, Gas};
use uint::construct_uint;

/// The amount of gas given to complete `vote` call.
pub const VOTE_GAS: Gas = Gas(100_000_000_000_000);

/// The amount of gas given to complete internal `on_stake_action` call.
pub const ON_STAKE_ACTION_GAS: Gas = Gas(20_000_000_000_000);

/// The amount of yocto NEAR the contract dedicates to guarantee that the "share" price never
/// decreases. It's used during rounding errors for share -> amount conversions.
pub const STAKE_SHARE_PRICE_GUARANTEE_FUND: Balance = 1_000_000_000_000;

/// There is no deposit balance attached.
pub const NO_DEPOSIT: Balance = 0;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl RewardFeeFraction {
    pub fn assert_valid(&self) {
        assert_ne!(self.denominator, 0, "Denominator must be a positive number");
        assert!(
            self.numerator <= self.denominator,
            "The reward fee must be less or equal to 1"
        );
    }

    pub fn multiply(&self, value: Balance) -> Balance {
        (U256::from(self.numerator) * U256::from(value) / U256::from(self.denominator)).as_u128()
    }
}