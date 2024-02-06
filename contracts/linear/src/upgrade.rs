// use crate::legacy::*;
use self::legacy::ContractV1_3_0;
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
        let contract: ContractV1_3_0 = env::state_read().expect("ERR_NOT_INITIALIZED");
        Self {
            owner_id: contract.owner_id,
            managers: contract.managers,
            treasury_id: contract.treasury_id,
            total_share_amount: contract.total_share_amount,
            total_staked_near_amount: contract.total_staked_near_amount,
            accounts: contract.accounts,
            paused: contract.paused,
            account_storage_usage: contract.account_storage_usage,
            beneficiaries: contract.beneficiaries,
            validator_pool: contract.validator_pool,
            whitelist_account_id: contract.whitelist_account_id,
            epoch_requested_stake_amount: contract.epoch_requested_stake_amount,
            epoch_requested_unstake_amount: contract.epoch_requested_unstake_amount,
            stake_amount_to_settle: contract.stake_amount_to_settle,
            unstake_amount_to_settle: contract.unstake_amount_to_settle,
            last_settlement_epoch: contract.last_settlement_epoch,
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
    /// Gas for calling `get_summary` method
    pub const GAS_FOR_GET_SUMMARY_CALL: Gas = Gas(15 * TGAS);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub fn upgrade() {
        env::setup_panic_hook();
        let contract: LiquidStakingContract =
            env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().as_bytes().to_vec();
        let migrate_method_name = b"migrate".to_vec();
        let get_summary_method_name = b"get_summary".to_vec();
        unsafe {
            // Load input (wasm code) into register 0.
            sys::input(0);
            // Create batch action promise for the current contract ID
            let promise_id =
                sys::promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
            // 1st batch action in the Tx: "deploy contract" (code is taken from register 0)
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd batch action in the Tx: call `migrate()` in the contract with sufficient gas
            let required_gas =
                env::used_gas() + GAS_FOR_COMPLETING_UPGRADE_CALL + GAS_FOR_GET_SUMMARY_CALL;
            require!(
                env::prepaid_gas() >= required_gas + MIN_GAS_FOR_MIGRATE_CALL,
                "Not enough gas to complete contract state migration"
            );
            let migrate_attached_gas = env::prepaid_gas() - required_gas;
            sys::promise_batch_action_function_call(
                promise_id,
                migrate_method_name.len() as _,
                migrate_method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                migrate_attached_gas.0,
            );
            // 3rd batch action in the Tx: call `get_summary()` in the contract to validate
            // the contract state. If the validation failed, the entire `upgrade()` method
            // will be rolled back. The `get_summary()` view call will access most of the
            // states in the contract, so should guarantee the contract is working as expected
            sys::promise_batch_action_function_call(
                promise_id,
                get_summary_method_name.len() as _,
                get_summary_method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                GAS_FOR_GET_SUMMARY_CALL.0,
            );
            sys::promise_return(promise_id);
        }
    }
}
