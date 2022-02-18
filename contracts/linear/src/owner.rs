use crate::*;
use near_sdk::{
    near_bindgen,
};
use std::collections::HashMap;

#[near_bindgen]
impl LiquidStakingContract {
    pub fn get_beneficiaries(& self) -> HashMap<AccountId, Fraction> {
        self.assert_owner();
        self.internal_get_beneficiaries()
    }

    pub fn set_beneficiary(
        &mut self,
        account_id: AccountId,
        fraction: Fraction
    ) {
        self.assert_owner();
        self.beneficiaries.insert(&account_id, &fraction);
    }

    pub fn remove_beneficiary(
        &mut self,
        account_id: AccountId
    ) {
        self.assert_owner();
        self.beneficiaries.remove(&account_id);
    }
}
