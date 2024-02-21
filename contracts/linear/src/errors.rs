// initialization
pub const ERR_ALREADY_INITIALZED: &str = "Already initialized";
pub const ERR_ACCOUNT_STAKING_WHILE_INIT: &str =
    "The current account has staking balance while initialization";
pub const ERR_NO_ENOUGH_INIT_DEPOSIT: &str =
    "The account doesn't have enough balance for initialization";

// owner
pub const ERR_NOT_OWNER: &str = "Only owner can perform this action";
pub const ERR_PAUSED: &str = "The contract is paused now. Please try later";
pub const ERR_ALREADY_PAUSED: &str = "The contract is already paused";
pub const ERR_NOT_PAUSED: &str = "The contract is not paused yet";

// manager
pub const ERR_NOT_MANAGER: &str = "Only manager can perform this action";

// account
#[allow(dead_code)]
pub const ERR_UNREGISTER_POSITIVE_UNSTAKED: &str = "Cannot delete the account because the unstaked amount is not empty. Withdraw your balance first.";

// fraction
pub const ERR_FRACTION_BAD_DENOMINATOR: &str = "Denominator cannot be zero";
pub const ERR_FRACTION_BAD_NUMERATOR: &str = "Numerator must <= denominator";
pub const ERR_BPS_SUM_ONE: &str = "bps sum should be less than 1";

// beneficiary
pub const ERR_TOO_MANY_BENEFICIARIES: &str = "Too many beneficiaries";

// stake
pub const ERR_NON_POSITIVE_STAKING_AMOUNT: &str = "Staking amount should be positive";
pub const ERR_NON_POSITIVE_CALCULATED_STAKING_SHARE: &str =
    "The calculated number of \"stake\" shares received for staking should be positive";
pub const ERR_NON_POSITIVE_CALCULATED_STAKED_AMOUNT: &str = "Invariant violation. Calculated staked amount must be positive, because \"stake\" share price should be at least 1";
pub const ERR_NO_ENOUGH_UNSTAKED_BALANCE: &str = "Not enough unstaked balance to stake";
pub const ERR_NO_ENOUGH_WITHDRAW_BALANCE: &str = "No enough unstaked balance to withdraw";

// unstake
pub const ERR_NON_POSITIVE_UNSTAKING_AMOUNT: &str = "Unstaking amount should be positive";
pub const ERR_NON_POSITIVE_CALCULATED_UNSTAKING_SHARE: &str = "Invariant violation. The calculated number of \"stake\" shares for unstaking should be positive";
pub const ERR_NO_ENOUGH_STAKED_BALANCE: &str = "Not enough staked balance to unstake";
pub const ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE: &str = "The total staked balance can't be 0";
pub const ERR_NON_POSITIVE_TOTAL_STAKE_SHARES: &str = "The total number of stake shares can't be 0";
pub const ERR_CONTRACT_NO_STAKED_BALANCE: &str = "Invariant violation. The calculated number of \"stake\" shares for unstaking should be positive";

// drain operations
pub const ERR_NON_ZERO_WEIGHT: &str = "Validator weight must be zero for drain operation";
pub const ERR_NON_ZERO_BASE_STAKE_AMOUNT: &str =
    "Validator base stake amount must be zero for drain operation";
pub const ERR_BAD_UNSTAKED_AMOUNT: &str = "Validator unstaked amount too large for drain unstake";
pub const ERR_NON_ZERO_STAKED_AMOUNT: &str =
    "Validator staked amount must be zero when drain withdraw";
pub const ERR_DRAINING: &str = "Validator is currently in draining process";
pub const ERR_NOT_IN_DRAINING: &str =
    "Validator is not in draining process. Cannot run drain withdraw";

// withdraw
pub const ERR_NON_POSITIVE_WITHDRAWAL_AMOUNT: &str = "Withdrawal amount should be positive";
pub const ERR_NO_ENOUGH_UNSTAKED_BALANCE_TO_WITHDRAW: &str =
    "Not enough unstaked balance to withdraw";
pub const ERR_UNSTAKED_BALANCE_NOT_AVAILABLE: &str =
    "The unstaked balance is not yet available due to unstaking delay";
pub const ERR_NO_ENOUGH_CONTRACT_BALANCE: &str =
    "No enough balance in contract to perform withdraw";

// validator
pub const ERR_MIN_RESERVE: &str = "Contract min reserve error";
pub const ERR_VALIDATOR_NOT_EXIST: &str = "Validator not exist";
pub const ERR_VALIDATOR_ALREADY_EXIST: &str = "Validator already exists";
pub const ERR_VALIDATOR_IN_USE: &str = "Validator is in use, cannot remove";
pub const ERR_NO_ENOUGH_GAS: &str = "No enough gas";
pub const ERR_BAD_VALIDATOR_LIST: &str = "Bad validator list";
pub const ERR_VALIDATOR_NOT_WHITELISTED: &str = "Validator not whitelisted";
pub const ERR_VALIDATOR_WHITELIST_NOT_SET: &str = "Validator whitelist not set";

pub const ERR_VALIDATOR_UNSTAKE_AMOUNT: &str = "No enough amount to unstake from validator";
pub const ERR_VALIDATOR_UNSTAKE_WHEN_LOCKED: &str =
    "Cannot unstake from a pending release validator";
pub const ERR_VALIDATOR_WITHDRAW_WHEN_LOCKED: &str =
    "Cannot withdraw from a pending release validator";
pub const ERR_VALIDATOR_ALREADY_EXECUTING_ACTION: &str = "Validator is already executing action";
pub const ERR_VALIDATOR_SYNC_BALANCE_NOT_EXPECTED: &str =
    "Validator sync balance is expected to be called after stake or unstake";
