use crate::legacy::*;
use crate::*;

#[near_bindgen]
impl LiquidStakingContract {
    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have changed state, you need to implement migration from old state (keep the old
    /// struct with different name to deserialize it first).
    /// After migration goes live, revert back to this implementation for next updates.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let mut contract: ContractV1_0_0 = env::state_read().expect("ERR_NOT_INITIALIZED");

        let old_beneficiaries = contract.beneficiaries.to_vec();
        contract.beneficiaries.clear();
        let mut new_beneficiaries = UnorderedMap::new(StorageKey::Beneficiaries);

        for (account, fraction) in old_beneficiaries {
            // current beneficiaries denominators are all 10_000,
            // so we can directly take its numerator
            require!(fraction.denominator == FULL_BASIS_POINTS);
            new_beneficiaries.insert(&account, &fraction.numerator);
        }

        Self {
            owner_id: contract.owner_id,
            managers: contract.managers,
            treasury_id: contract.treasury_id,
            total_share_amount: contract.total_share_amount,
            total_staked_near_amount: contract.total_staked_near_amount,
            accounts: contract.accounts,
            paused: contract.paused,
            account_storage_usage: contract.account_storage_usage,
            beneficiaries: new_beneficiaries, // migrate
            liquidity_pool: contract.liquidity_pool,
            validator_pool: contract.validator_pool,
            whitelist_account_id: None, // migrate
            epoch_requested_stake_amount: contract.epoch_requested_stake_amount,
            epoch_requested_unstake_amount: contract.epoch_requested_unstake_amount,
            stake_amount_to_settle: contract.stake_amount_to_settle,
            unstake_amount_to_settle: contract.unstake_amount_to_settle,
            last_settlement_epoch: contract.last_settlement_epoch,
            farms: contract.farms,
            active_farms: contract.active_farms,
            authorized_farm_tokens: contract.authorized_farm_tokens,
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::Gas;
    use near_sys as sys;

    use super::*;

    /// Gas for completing the upgrade call
    pub const GAS_FOR_COMPLETING_UPGRADE_CALL: Gas = Gas(10 * TGAS);
    /// Minimum gas for calling state migration call. Please notice the gas cost will be higher
    /// if the number of accounts and validator pools grows.
    pub const MIN_GAS_FOR_MIGRATE_CALL: Gas = Gas(10 * TGAS);

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
            let required_gas = env::used_gas() + GAS_FOR_COMPLETING_UPGRADE_CALL;
            require!(
                env::prepaid_gas() >= required_gas + MIN_GAS_FOR_MIGRATE_CALL,
                "Not enough gas to complete contract state migration"
            );
            let migrate_attached_gas = env::prepaid_gas() - required_gas;
            sys::promise_batch_action_function_call(
                promise_id,
                method_name.len() as _,
                method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                migrate_attached_gas.0,
            );
        }
    }
}
