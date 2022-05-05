// initialization
pub const ERR_ALREADY_INITIALZED: &str = "Already initialized";
pub const ERR_ACCOUNT_STAKING_WHILE_INIT: &str =
    "The current account has staking balance while initialization";
pub const ERR_NO_ENOUGH_INIT_DEPOSIT: &str =
    "The account doesn't have enough balance for initialization";

// owner
pub const ERR_NOT_OWNER: &str = "Only owner can perform this action";

// manager
pub const ERR_NOT_MANAGER: &str = "Only manager can perform this action";

// account
pub const ERR_NO_ACCOUNT: &str = "Account not found";
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

// sync balance
pub const ERR_SYNC_BALANCE_BAD_TOTAL: &str =
    "Diff between new total balance and old balance > 1 yN";
pub const ERR_SYNC_BALANCE_BAD_STAKED: &str =
    "Diff between new staked balance and old balance too large";
pub const ERR_SYNC_BALANCE_BAD_UNSTAKED: &str =
    "Diff between new unstaked balance and old balance too large";

// drain operations
pub const ERR_NON_ZERO_WEIGHT: &str = "Validator weight must be zero for drain operation";
pub const ERR_NON_ZERO_UNSTAKED_AMOUNT: &str =
    "Validator unstaked amount must be zero when drain unstake";
pub const ERR_NON_ZERO_STAKED_AMOUNT: &str =
    "Validator staked amount must be zero when drain withdraw";

// withdraw
pub const ERR_NON_POSITIVE_WITHDRAWAL_AMOUNT: &str = "Withdrawal amount should be positive";
pub const ERR_NO_ENOUGH_UNSTAKED_BALANCE_TO_WITHDRAW: &str =
    "Not enough unstaked balance to withdraw";
pub const ERR_UNSTAKED_BALANCE_NOT_AVAILABLE: &str =
    "The unstaked balance is not yet available due to unstaking delay";
pub const ERR_INCONSISTANT_BALANCE: &str = "Contract balance less than liquidity pool balance";
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

// liquidity pool
pub const ERR_NON_POSITIVE_MIN_FEE: &str = "The min fee basis points should be positive";
pub const ERR_FEE_MAX_LESS_THAN_MIN: &str =
    "The max fee basis points should be no less than the min fee";
pub const ERR_FEE_EXCEEDS_UP_LIMIT: &str = "The fee basis points should be less than 10000";
pub const ERR_NON_POSITIVE_EXPECTED_NEAR_AMOUNT: &str =
    "The expected NEAR amount should be positive";

pub const ERR_NON_POSITIVE_LIQUIDITY_POOL_SHARE: &str =
    "The calculated number of shares received for adding liquidity should be positive";
pub const ERR_NON_POSITIVE_MIN_RECEIVED_AMOUNT: &str =
    "The expected received NEAR amount should be positive";
pub const ERR_NON_POSITIVE_RECEIVED_FEE: &str =
    "The fee received by the liquidity pool should be positive";
pub const ERR_NO_ENOUGH_LIQUIDITY: &str = "Not enough liquidity in the pool";
pub const ERR_ACCOUNT_NO_SHARE: &str = "Account has no shares in liquidity pool";
pub const ERR_NO_ENOUGH_LIQUIDITY_SHARES_TO_REMOVE: &str =
    "Not enough liquidity shares to remove from the pool";
pub const ERR_NON_POSITIVE_REMOVE_LIQUIDITY_AMOUNT: &str =
    "The amount of value to be removed from liquidity pool should be positive";

// staking farm
pub const ERR_FARM_START_TOO_EARLY: &str =
    "Farm start date should not be earlier than the current time";
pub const ERR_FARM_END_TOO_EARLY: &str = "Farm end date is too close to start date";
pub const ERR_NON_POSITIVE_FARM_AMOUNT: &str = "Farm amount should be positive";
pub const ERR_FARM_AMOUNT_TOO_SMALL: &str = "Farm amount is too small";
pub const ERR_NO_FARM: &str = "Farm not found";
pub const ERR_NO_FARM_REWARDS: &str = "No rewards to claim";
pub const ERR_FARM_MSG_WRONG_FORMAT: &str = "Farm message format is wrong";
pub const ERR_NOT_AUTHORIZED_TOKEN: &str = "Token has not been authorized";
pub const ERR_NOT_AUTHORIZED_USER: &str = "The sender has not been authorized";
pub const ERR_TOO_MANY_ACTIVE_FARMS: &str = "Too many active farms";

pub const ERR_GET_OWNER_NO_RESULT: &str = "get_owner must have result";
pub const ERR_PARSE_OWNER_ID: &str = "Failed to parse owner ID";
pub const ERR_CALLER_NOT_OWNER: &str = "Caller is not an owner";
