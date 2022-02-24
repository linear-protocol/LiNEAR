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
  
   /// Add authorized user to the current contract.
//    pub fn add_authorized_user(&mut self, account_id: AccountId) {
//         self.assert_owner();
//         self.authorized_users.insert(&account_id);
//     }

    /// Remove authorized user from the current contract.
    // pub fn remove_authorized_user(&mut self, account_id: AccountId) {
    //     self.assert_owner();
    //     self.authorized_users.remove(&account_id);
    // }

    /// Add authorized token.
    pub fn add_authorized_farm_token(&mut self, token_id: &AccountId) {
        self.assert_owner();
        self.authorized_farm_tokens.insert(&token_id);
    }

    /// Remove authorized token.
    pub fn remove_authorized_farm_token(&mut self, token_id: &AccountId) {
        self.assert_owner();
        self.authorized_farm_tokens.remove(&token_id);
    }

    // Asserts that the method was called by the owner or authorized user.
    // pub(crate) fn assert_owner_or_authorized_user(&self) {
    //     let account_id = env::predecessor_account_id();
    //     assert!(
    //         account_id == self.get_owner_id()
    //             || self.authorized_users.contains(&account_id),
    //         "ERR_NOT_AUTHORIZED_USER"
    //     );
    // }
}
