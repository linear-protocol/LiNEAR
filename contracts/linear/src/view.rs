use crate::*;
use near_sdk::{
    near_bindgen,
};
use std::collections::HashMap;

/// public view functions
#[near_bindgen]
impl LiquidStakingContract {
    pub fn get_total_share_amount(& self) -> ShareBalance {
        self.total_share_amount
    }

    pub fn get_total_staked_near_amount(& self) -> Balance {
        self.total_staked_near_amount
    }

    pub fn get_beneficiaries(& self) -> HashMap<AccountId, Fraction> {
        self.internal_get_beneficiaries()
    }
}
