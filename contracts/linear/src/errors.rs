pub const ERR_ALREADY_INITIALZED: &str = "Already initialized";
pub const ERR_ACCOUNT_STAKING_WHILE_INIT: &str = "The current account has staking balance while initialization";
pub const ERR_NO_ENOUGH_INIT_DEPOSIT: &str = "The account doesn't have enough balance for initialization";

pub const ERR_FRACTION_BAD_DENOMINATOR: &str = "Denominator cannot be zero";
pub const ERR_FRACTION_BAD_NUMERATOR: &str = "Numerator must <= denominator";

pub const ERR_CALL_DEPOSIT: &str = "Deposit is not supported, please use deposit_and_stake";
// stake
pub const ERR_NON_POSITIVE_STAKING_AMOUNT: &str = "Staking amount should be positive";
pub const ERR_NON_POSITIVE_CALCULATED_STAKING_SHARE: &str = "The calculated number of \"stake\" shares received for staking should be positive";
pub const ERR_NON_POSITIVE_CALCULATED_STAKED_AMOUNT: &str = "Invariant violation. Calculated staked amount must be positive, because \"stake\" share price should be at least 1";
// unstake
pub const ERR_NO_ENOUGH_UNSTAKED_BALANCE: &str = "Not enough unstaked balance to stake";
pub const ERR_NON_POSITIVE_UNSTAKING_AMOUNT: &str = "Unstaking amount should be positive";
pub const ERR_NON_POSITIVE_CALCULATED_UNSTAKING_SHARE: &str = "Invariant violation. The calculated number of \"stake\" shares for unstaking should be positive";
pub const ERR_NO_ENOUGH_STAKED_BALANCE: &str = "Not enough staked balance to unstake";
pub const ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE: &str = "The total staked balance can't be 0";
pub const ERR_CONTRACT_NO_STAKED_BALANCE: &str = "Invariant violation. The calculated number of \"stake\" shares for unstaking should be positive";

pub const ERR_MIN_RESERVE: &str = "Contract min reserve error";
pub const ERR_VALIDATOR_NOT_EXIST: &str = "Validator not exist";
