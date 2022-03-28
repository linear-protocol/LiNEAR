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
        self.assert_owner();
        self.internal_add_manager(&new_manager_id);
    }

    pub fn remove_manager(&mut self, manager_id: AccountId) -> bool {
        self.assert_owner();
        self.internal_remove_manager(&manager_id)
    }

    pub fn set_beneficiary(&mut self, account_id: AccountId, fraction: Fraction) {
        self.assert_owner();
        fraction.assert_valid();

        if self.beneficiaries.len() == MAX_BENEFICIARIES &&
            self.beneficiaries.get(&account_id).is_none() {
            env::panic_str(ERR_TOO_MANY_BENEFICIARIES);
        }

        let fraction_sum = self.beneficiaries.values().map(|f| {
            f.as_f32()
        }).reduce(|sum, v| {
            sum + v
        });
        let fraction_sum = fraction_sum.unwrap_or_default();

        let old_value = self.beneficiaries.get(&account_id)
            .map(|f| {f.as_f32()})
            .unwrap_or_default();

        require!(
            fraction_sum - old_value + fraction.as_f32() <= 1.0,
            ERR_FRACTION_SUM_ONE
        );

        self.beneficiaries.insert(&account_id, &fraction);
    }

    pub fn remove_beneficiary(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.beneficiaries.remove(&account_id);
    }

    /// Set account ID of the treasury
    pub fn set_treasury(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.treasury_id = account_id;
    }

    // --- Staking Farm ----

    /// Add authorized user to the current contract.
    // pub fn add_authorized_user(&mut self, account_id: AccountId) {
    //     self.assert_owner();
    //     self.authorized_users.insert(&account_id);
    // }

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

    // --- Liquidity Pool ----

    pub fn configure_liquidity_pool(&mut self, config: LiquidityPoolConfig) {
        self.assert_owner();
        self.liquidity_pool.configure(config);
    }

    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have changed state, you need to implement migration from old state (keep the old struct with different name to deserialize it first).
    /// After migration goes live, revert back to this implementation for next updates.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let contract: LiquidStakingContract = env::state_read().expect("ERR_NOT_INITIALIZED");
        contract
    }
}

#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::Gas;
    use near_sys as sys;

    use super::*;

    /// Gas for calling migration call.
    pub const GAS_FOR_MIGRATE_CALL: Gas = Gas(5_000_000_000_000);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub fn upgrade() {
        env::setup_panic_hook();
        let contract: LiquidStakingContract =
            env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().as_bytes().to_vec();
        let method_name = "migrate".as_bytes().to_vec();
        unsafe {
            // Load input (wasm code) into register 0.
            sys::input(0);
            // Create batch action promise for the current contract ID
            let promise_id =
                sys::promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
            // 1st action in the Tx: "deploy contract" (code is taken from register 0)
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd action in the Tx: call this_contract.migrate() with remaining gas
            let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_MIGRATE_CALL;
            sys::promise_batch_action_function_call(
                promise_id,
                method_name.len() as _,
                method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                attached_gas.0,
            );
        }
    }
}
