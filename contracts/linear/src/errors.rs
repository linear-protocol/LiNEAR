pub const ERR_ALREADY_INITIALZED: &str = "Already initialized";
pub const ERR_ACCOUNT_STAKING_WHILE_INIT: &str = "The current account has staking balance while initialization";
pub const ERR_NO_ENOUGH_INIT_DEPOSIT: &str = "The account doesn't have enough balance for initialization";

pub const ERR_NOT_OWNER: &str = "Only owner can perform this action";

// fraction
pub const ERR_FRACTION_BAD_DENOMINATOR: &str = "Denominator cannot be zero";
pub const ERR_FRACTION_BAD_NUMERATOR: &str = "Numerator must <= denominator";

// stake
pub const ERR_NON_POSITIVE_STAKING_AMOUNT: &str = "Staking amount should be positive";
pub const ERR_NON_POSITIVE_CALCULATED_STAKING_SHARE: &str = "The calculated number of \"stake\" shares received for staking should be positive";
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

// withdraw
pub const ERR_NON_POSITIVE_WITHDRAWAL_AMOUNT: &str = "Withdrawal amount should be positive";
pub const ERR_NO_ENOUGH_UNSTAKED_BALANCE_TO_WITHDRAW: &str = "Not enough unstaked balance to withdraw";
pub const ERR_UNSTAKED_BALANCE_NOT_AVAILABLE: &str = "The unstaked balance is not yet available due to unstaking delay";

// validator
pub const ERR_MIN_RESERVE: &str = "Contract min reserve error";
pub const ERR_VALIDATOR_NOT_EXIST: &str = "Validator not exist";
pub const ERR_VALIDATOR_ALREADY_EXIST: &str = "Validator already exists";
pub const ERR_VALIDATOR_IN_USE: &str = "Validator is in use, cannot remove";
pub const ERR_NO_ENOUGH_GAS: &str = "No enough gas";
pub const ERR_BAD_VALIDATOR_LIST: &str = "Bad validator list";

pub const ERR_VALIDATOR_UNSTAKE_AMOUNT: &str = "No enough amount to unstake from validator";
pub const ERR_VALIDATOR_UNSTAKE_WHEN_LOCKED: &str = "Cannot unstake from a pending release validator";
pub const ERR_VALIDATOR_WITHDRAW_WHEN_LOCKED: &str = "Cannot withdraw from a pending release validator";

// liquidity pool
pub const ERR_NON_POSITIVE_LIQUIDITY_POOL_SHARE: &str = "The calculated number of shares received for adding liquidity should be positive";
pub const ERR_NON_POSITIVE_MIN_RECEIVED_AMOUNT: &str = "The expected received NEAR amount should be positive";
pub const ERR_NON_POSITIVE_MIN_FEE: &str = "The min fee should be positive";
pub const ERR_FEE_MAX_LESS_THAN_MIN: &str = "The max fee should be no less than the min fee";
pub const ERR_FEE_EXCEEDS_UP_LIMIT: &str = "The fee should be less than 10000";
pub const ERR_NO_ENOUGH_LIQUIDITY: &str = "Not enough liquidity in the pool";
pub const ERR_ACCOUNT_NO_SHARE: &str = "Account has no shares in liquidity pool";
pub const ERR_NO_ENOUGH_LIQUIDITY_SHARES_TO_REMOVE: &str = "Not enough liquidity shares to remove from the pool";
