use near_sdk::{
    ext_contract, AccountId, Balance, EpochHeight, env,
    json_types::{U128},
};

const NUM_EPOCHS_TO_UNLOCK: EpochHeight = 4;

#[ext_contract(ext_staking_pool)]
pub trait ExtStakingPool {
    fn get_account_staked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_total_balance(&self, account_id: AccountId) -> U128;

    fn deposit(&mut self);

    fn deposit_and_stake(&mut self);

    fn withdraw(&mut self, amount: U128);

    fn withdraw_all(&mut self);

    fn stake(&mut self, amount: U128);

    fn unstake(&mut self, amount: U128);

    fn unstake_all(&mut self);
}

/// struct for staking pool validator
pub struct Validator {
    account_id: AccountId,

    staked_amount: Balance,
    unstaked_amount: Balance,

    /// the epoch num when latest unstake action happened on this validator
    unstake_fired_epoch: EpochHeight,
}

impl Validator {
    pub fn new(
        account_id: AccountId,
    ) -> Self {
        Self {
            account_id,
            staked_amount: 0,
            unstaked_amount: 0,
            unstake_fired_epoch: 0,
        }
    }

    pub fn pending_release(& self) -> bool {
        let current_epoch = env::epoch_height();
        current_epoch >= self.unstake_fired_epoch &&
            current_epoch < self.unstake_fired_epoch + NUM_EPOCHS_TO_UNLOCK
    }
}
