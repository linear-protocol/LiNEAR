use near_sdk::{
    env, EpochHeight, near_bindgen, BorshStorageKey
};
use crate::*;

pub fn assert_is_callback() {
    assert_eq!(env::predecessor_account_id(), env::current_account_id());
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

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    Shares,
    Validators,
}