use near_sdk::{
    env, EpochHeight,
};
#[cfg(test)]
use crate::*;

pub fn assert_is_callback() {
    assert_eq!(env::predecessor_account_id(), env::current_account_id());
}

#[cfg(not(test))]
pub fn get_epoch_height() -> EpochHeight {
    env::epoch_height()
}

#[cfg(test)]
pub fn get_epoch_height() -> EpochHeight {
    let test_epoch_height_key: &[u8] = "_test_epoch_".as_bytes();
    let raw_epoch_option = env::storage_read(test_epoch_height_key);

    if let Some(raw_epoch) = raw_epoch_option {
        EpochHeight::try_from_slice(&raw_epoch).unwrap_or(0)
    } else {
        0
    }
}

#[cfg(test)]
#[near_bindgen]
impl LiquidStakingContract {
    pub fn set_epoch_height(epoch: EpochHeight) {
        let test_epoch_height_key: &[u8] = "_test_epoch_".as_bytes();
        env::storage_write(test_epoch_height_key, &epoch.try_to_vec().unwrap_or_default());
    }
}
