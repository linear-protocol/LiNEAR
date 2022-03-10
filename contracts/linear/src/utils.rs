use near_sdk::{
    env, EpochHeight, near_bindgen, BorshStorageKey
};
use crate::*;

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

/// Epoch height helper methods only available for testing
#[near_bindgen]
impl LiquidStakingContract {
    #[cfg(feature = "test")]
    pub fn set_epoch_height(&mut self, epoch: EpochHeight) {
        let test_epoch_height_key: &[u8] = "_test_epoch_".as_bytes();
        env::storage_write(test_epoch_height_key, &epoch.try_to_vec().unwrap_or_default());
    }

    #[cfg(feature = "test")]
    pub fn read_epoch_height(&self) -> EpochHeight {
        get_epoch_height()
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
pub (crate) fn num_shares_from_staked_amount_rounded_down(amount: Balance, context: &Context) -> ShareBalance {
    require!(context.total_staked_near_amount > 0, ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE);
    (U256::from(context.total_share_amount) * U256::from(amount)
        / U256::from(context.total_staked_near_amount))
    .as_u128()
}

/// Returns the staked amount rounded down corresponding to the given number of "stake" shares.
pub (crate) fn staked_amount_from_num_shares_rounded_down(num_shares: ShareBalance, context: &Context) -> Balance {
    require!(context.total_share_amount > 0, ERR_NON_POSITIVE_TOTAL_STAKE_SHARES);
    (U256::from(context.total_staked_near_amount) * U256::from(num_shares)
        / U256::from(context.total_share_amount))
    .as_u128()
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    Shares,
    Beneficiaries,
    Validators,
    Farms,
    // AuthorizedUsers,
    AuthorizedFarmTokens
}
