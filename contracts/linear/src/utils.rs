use near_sdk::{
    env,
};

pub fn assert_is_callback() {
    assert_eq!(env::predecessor_account_id(), env::current_account_id());
}
