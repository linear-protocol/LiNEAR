use crate::*;
use near_sdk::{
    near_bindgen,
};

#[near_bindgen]
impl LiquidStakingContract {
    pub fn set_beneficiary(
        &mut self,
        account_id: AccountId,
        fraction: Fraction
    ) {
        self.assert_owner();
        fraction.assert_valid();
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
