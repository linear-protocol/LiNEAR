use crate::*;
use near_sdk::{json_types::U128, near_bindgen, AccountId};
use std::collections::HashMap;

/// The human readable summary of the liquid staking contract
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Summary {
    /// Total amount of LiNEAR that was minted (minus burned).
    pub total_share_amount: U128,
    /// Total amount of NEAR that was staked by users to this contract.
    pub total_staked_near_amount: U128,

    /// LiNEAR price
    pub ft_price: U128,

    /// Number of nodes in validator pool
    pub validators_num: u64,

    /// Amount of NEAR that needs to be settled by staking on validators
    pub stake_amount_to_settle: U128,
    /// Amount of NEAR that needs to be settled by unstaking from validators
    pub unstake_amount_to_settle: U128,
    /// Total base stake amount of NEAR on validators
    pub validators_total_base_stake_amount: U128,
    /// Amount of NEAR that is requested to stake by all users during the last epoch
    pub epoch_requested_stake_amount: U128,
    /// Amount of NEAR that is requested to unstake by all users during the last epoch
    pub epoch_requested_unstake_amount: U128,
}

/// public view functions
#[near_bindgen]
impl LiquidStakingContract {
    pub fn get_total_share_amount(&self) -> U128 {
        self.total_share_amount.into()
    }

    pub fn get_beneficiaries(&self) -> HashMap<AccountId, u32> {
        self.internal_get_beneficiaries()
    }

    pub fn get_managers(&self) -> Vec<AccountId> {
        self.internal_get_managers()
    }

    pub fn get_summary(&self) -> Summary {
        Summary {
            total_share_amount: self.total_share_amount.into(),
            total_staked_near_amount: self.total_staked_near_amount.into(),
            ft_price: self.ft_price(),
            validators_num: self.validator_pool.count(),
            stake_amount_to_settle: self.stake_amount_to_settle.into(),
            unstake_amount_to_settle: self.unstake_amount_to_settle.into(),
            validators_total_base_stake_amount: self.validator_pool.total_base_stake_amount.into(),
            epoch_requested_stake_amount: self.epoch_requested_stake_amount.into(),
            epoch_requested_unstake_amount: self.epoch_requested_unstake_amount.into(),
        }
    }

    /// Return account details for staking
    pub fn get_account_details(&self, account_id: AccountId) -> AccountDetailsView {
        let account = self.internal_get_account(&account_id);
        AccountDetailsView {
            account_id,
            unstaked_balance: account.unstaked.into(),
            staked_balance: self
                .staked_amount_from_num_shares_rounded_down(account.stake_shares)
                .into(),
            unstaked_available_epoch_height: account.unstaked_available_epoch_height,
            can_withdraw: account.unstaked_available_epoch_height <= get_epoch_height(),
        }
    }

    // --- Staking Pool view methods ---

    /// Returns the unstaked balance of the given account.
    pub fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128 {
        self.get_account(account_id).unstaked_balance
    }

    /// Returns the staked balance of the given account.
    /// NOTE: This is computed from the amount of "stake" shares the given account has and the
    /// current amount of total staked balance and total stake shares on the account.
    pub fn get_account_staked_balance(&self, account_id: AccountId) -> U128 {
        self.get_account(account_id).staked_balance
    }

    /// Returns the total balance of the given account (including staked and unstaked balances).
    pub fn get_account_total_balance(&self, account_id: AccountId) -> U128 {
        let account = self.get_account(account_id);
        (account.unstaked_balance.0 + account.staked_balance.0).into()
    }

    /// Returns `true` if the given account can withdraw tokens in the current epoch.
    pub fn is_account_unstaked_balance_available(&self, account_id: AccountId) -> bool {
        self.get_account(account_id).can_withdraw
    }

    /// Returns the total staking balance.
    pub fn get_total_staked_balance(&self) -> U128 {
        self.total_staked_near_amount.into()
    }

    /// Returns account ID of the staking pool owner.
    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    /// Returns the current reward fee as a fraction.
    /// Weighted average fee of all validators: 4.9%, treasury fee 1%.
    /// Total fee: 100% - (100% - 4.9%) * (100% - 1%)
    pub fn get_reward_fee_fraction(&self) -> Fraction {
        Fraction {
            numerator: 58,
            denominator: 1000,
        }
    }

    /// Returns the staking public key
    pub fn get_staking_key(&self) -> PublicKey {
        panic!("no need to specify public key for liquid staking pool");
    }

    /// Returns true if the staking is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns human readable representation of the account for the given account ID.
    pub fn get_account(&self, account_id: AccountId) -> HumanReadableAccount {
        let account = self.internal_get_account(&account_id);
        HumanReadableAccount {
            account_id,
            unstaked_balance: account.unstaked.into(),
            staked_balance: self
                .staked_amount_from_num_shares_rounded_down(account.stake_shares)
                .into(),
            can_withdraw: account.unstaked_available_epoch_height <= get_epoch_height(),
        }
    }

    /// Returns the number of accounts that have positive balance on this staking pool.
    pub fn get_number_of_accounts(&self) -> u64 {
        self.accounts.len()
    }

    /// Returns the list of accounts
    pub fn get_accounts(&self, from_index: u64, limit: u64) -> Vec<HumanReadableAccount> {
        let keys = self.accounts.keys_as_vector();

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| self.get_account(keys.get(index).unwrap()))
            .collect()
    }

    // --- custom staking pool view methods ---

    /// confirm if the user can perform withdraw now
    pub fn can_account_withdraw(&self, account_id: AccountId, amount: U128) {
        self.assert_can_withdraw(&account_id, amount.0);
    }
}
