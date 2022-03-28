use crate::*;
use near_sdk::{env, near_bindgen, EpochHeight};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Fraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl Fraction {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        let f = Self {
            numerator,
            denominator,
        };
        f.assert_valid();
        return f;
    }

    pub fn assert_valid(&self) {
        require!(self.denominator != 0, ERR_FRACTION_BAD_DENOMINATOR);
        require!(
            self.numerator <= self.denominator,
            ERR_FRACTION_BAD_NUMERATOR
        );
    }

    pub fn multiply(&self, value: u128) -> u128 {
        (U256::from(self.numerator) * U256::from(value) / U256::from(self.denominator)).as_u128()
    }

    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }
}

#[cfg(not(feature = "test"))]
pub fn get_epoch_height() -> EpochHeight {
    env::epoch_height()
}

#[cfg(feature = "test")]
pub fn get_epoch_height() -> EpochHeight {
    let test_epoch_height_key: &[u8] = "_test_epoch_".as_bytes();
    let raw_epoch_option = env::storage_read(test_epoch_height_key);

    // default epoch is 10 for testing
    if let Some(raw_epoch) = raw_epoch_option {
        EpochHeight::try_from_slice(&raw_epoch).unwrap_or(10)
    } else {
        10
    }
}

#[near_bindgen]
impl LiquidStakingContract {
    /// Set epoch height helper method, only available for testing
    #[cfg(feature = "test")]
    pub fn set_epoch_height(&mut self, epoch: EpochHeight) {
        let test_epoch_height_key: &[u8] = "_test_epoch_".as_bytes();
        env::storage_write(
            test_epoch_height_key,
            &epoch.try_to_vec().unwrap_or_default(),
        );
    }

    /// Read epoch height helper method, only available for testing
    #[cfg(feature = "test")]
    pub fn read_epoch_height(&self) -> EpochHeight {
        get_epoch_height()
    }

    /// Add epoch rewards method, only available for testing
    #[cfg(feature = "test")]
    pub fn add_epoch_rewards(&mut self, amount: U128) {
        self.assert_owner();
        let amount: Balance = amount.into();
        require!(amount > 0, "Added rewards amount must be positive");
        self.total_staked_near_amount += amount;
    }
}

/// Returns the number of "stake" shares rounded down corresponding to the given staked balance
/// amount.
///
/// price = total_staked / total_shares
/// Price is fixed
/// (total_staked + amount) / (total_shares + num_shares) = total_staked / total_shares
/// (total_staked + amount) * total_shares = total_staked * (total_shares + num_shares)
/// amount * total_shares = total_staked * num_shares
/// num_shares = amount * total_shares / total_staked
pub(crate) fn num_shares_from_staked_amount_rounded_down(
    amount: Balance,
    context: &Context,
) -> ShareBalance {
    require!(
        context.total_staked_near_amount > 0,
        ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE
    );
    (U256::from(context.total_share_amount) * U256::from(amount)
        / U256::from(context.total_staked_near_amount))
    .as_u128()
}

/// Returns the staked amount rounded down corresponding to the given number of "stake" shares.
pub(crate) fn staked_amount_from_num_shares_rounded_down(
    num_shares: ShareBalance,
    context: &Context,
) -> Balance {
    require!(
        context.total_share_amount > 0,
        ERR_NON_POSITIVE_TOTAL_STAKE_SHARES
    );
    (U256::from(context.total_staked_near_amount) * U256::from(num_shares)
        / U256::from(context.total_share_amount))
    .as_u128()
}

/// The absolute diff between left and right is not greater than epsilon.
/// This is useful when user submit requests that approximaitely equal to the acount's NEAR/LiNEAR balance
pub(crate) fn abs_diff_eq(left: u128, right: u128, epsilon: u128) -> bool {
    return left <= right + epsilon && right <= left + epsilon;
}
