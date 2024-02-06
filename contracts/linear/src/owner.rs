use crate::*;
use near_sdk::near_bindgen;

const MAX_BENEFICIARIES: u64 = 10;

#[near_bindgen]
impl LiquidStakingContract {
    pub fn set_owner(&mut self, new_owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = new_owner_id;
    }

    pub fn add_manager(&mut self, new_manager_id: AccountId) {
        self.assert_running();
        self.assert_owner();
        self.internal_add_manager(&new_manager_id);
    }

    pub fn remove_manager(&mut self, manager_id: AccountId) -> bool {
        self.assert_running();
        self.assert_owner();
        self.internal_remove_manager(&manager_id)
    }

    pub fn set_beneficiary(&mut self, account_id: AccountId, bps: u32) {
        self.assert_running();
        self.assert_owner();

        if self.beneficiaries.len() == MAX_BENEFICIARIES
            && self.beneficiaries.get(&account_id).is_none()
        {
            env::panic_str(ERR_TOO_MANY_BENEFICIARIES);
        }

        let bps_sum = self.beneficiaries.values().reduce(|sum, v| sum + v);
        let bps_sum = bps_sum.unwrap_or_default();

        let old_value = self.beneficiaries.get(&account_id).unwrap_or_default();

        require!(
            bps_sum - old_value + bps <= FULL_BASIS_POINTS,
            ERR_BPS_SUM_ONE
        );

        self.beneficiaries.insert(&account_id, &bps);
    }

    pub fn remove_beneficiary(&mut self, account_id: AccountId) {
        self.assert_running();
        self.assert_owner();
        self.beneficiaries.remove(&account_id);
    }

    /// Set account ID of the treasury
    pub fn set_treasury(&mut self, account_id: AccountId) {
        self.assert_running();
        self.assert_owner();
        self.treasury_id = account_id;
    }

    /// Set whitelist account ID
    pub fn set_whitelist_contract_id(&mut self, account_id: AccountId) {
        self.assert_running();
        self.assert_owner();
        self.whitelist_account_id = Some(account_id);
    }

    // --- Pause ---

    pub fn pause(&mut self) {
        self.assert_owner();
        require!(!self.paused, ERR_ALREADY_PAUSED);
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.assert_owner();
        require!(self.paused, ERR_NOT_PAUSED);
        self.paused = false;
    }
}
