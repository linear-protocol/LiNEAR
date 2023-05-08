use crate::*;
use near_sdk::{
    json_types::{U128, U64},
    near_bindgen, AccountId,
};
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

    /// Target NEAR amount in Liquidity Pool
    pub lp_target_amount: U128,
    /// Current NEAR amount in Liquidity Pool
    pub lp_near_amount: U128,
    /// Current LiNEAR amount in Liquidity Pool
    pub lp_staked_share: U128,
    /// Current instant unstake fee in Liquidity Pool.
    /// For example, fee basis points is `30`, which means `0.3%` in percentage
    pub lp_swap_fee_basis_points: u32,
    /// Total received unstake fee as LiNEAR in Liquidity Pool
    pub lp_total_fee_shares: U128,

    /// Number of nodes in validator pool
    pub validators_num: u64,

    /// Active farms that affect stakers.
    /// Can calculate rate of return of this pool with farming by:
    /// `farm_reward_per_day = farms.iter().map(farms.amount / (farm.end_date - farm.start_date) / DAY_IN_NS * PRICES[farm.token_id]).sum()`
    /// `near_reward_per_day = total_near_emission_per_day * this.total_staked_near_amount / total_near_staked`
    /// `total_reward_per_day = farm_reward_per_day + near_reward_per_day * NEAR_PRICE`
    /// `reward_rate = total_reward_per_day / (this.total_staked_near_amount * NEAR_PRICE)`
    pub farms: Vec<HumanReadableFarm>,

    /// Amount of NEAR that needs to be settled by staking on validators
    pub stake_amount_to_settle: U128,
    /// Amount of NEAR that needs to be settled by unstaking from validators
    pub unstake_amount_to_settle: U128,
    pub epoch_requested_stake_amount: U128,
    pub epoch_requested_unstake_amount: U128,
    /// Total base stake amount of NEAR on validators
    pub validators_total_base_stake_amount: U128,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct HumanReadableFarm {
    pub farm_id: u64,
    pub name: String,
    pub token_id: AccountId,
    pub amount: U128,
    pub start_date: U64,
    pub end_date: U64,
    pub active: bool,
}

impl HumanReadableFarm {
    fn from(farm_id: u64, farm: Farm) -> Self {
        let active = farm.is_active();
        HumanReadableFarm {
            farm_id,
            name: farm.name,
            token_id: farm.token_id,
            amount: U128(farm.amount),
            start_date: U64(farm.start_date),
            end_date: U64(farm.end_date),
            active,
        }
    }
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
            lp_target_amount: self.liquidity_pool.config.expected_near_amount,
            lp_near_amount: self.liquidity_pool.amounts[0].into(),
            lp_staked_share: self.liquidity_pool.amounts[1].into(),
            lp_swap_fee_basis_points: self
                .liquidity_pool
                .get_current_swap_fee_basis_points(10 * ONE_NEAR),
            lp_total_fee_shares: self.liquidity_pool.total_fee_shares.into(),
            validators_num: self.validator_pool.count(),
            farms: self.get_active_farms(),
            stake_amount_to_settle: self.stake_amount_to_settle.into(),
            unstake_amount_to_settle: self.unstake_amount_to_settle.into(),
            epoch_requested_stake_amount: self.epoch_requested_stake_amount.into(),
            epoch_requested_unstake_amount: self.epoch_requested_unstake_amount.into(),
            validators_total_base_stake_amount: self.validator_pool.total_base_stake_amount.into(),
        }
    }

    // Return account details including staking pool and liquidity pool
    pub fn get_account_details(&self, account_id: AccountId) -> AccountDetailsView {
        let account = self.internal_get_account(&account_id);
        AccountDetailsView {
            account_id: account_id.clone(),
            unstaked_balance: account.unstaked.into(),
            staked_balance: self
                .staked_amount_from_num_shares_rounded_down(account.stake_shares)
                .into(),
            unstaked_available_epoch_height: account.unstaked_available_epoch_height,
            can_withdraw: account.unstaked_available_epoch_height <= get_epoch_height(),
            liquidity_pool_share: self.liquidity_pool.get_account_shares(&account_id).into(),
            liquidity_pool_share_value: self
                .liquidity_pool
                .get_account_value(&account_id, &self.internal_get_context())
                .into(),
            liquidity_pool_share_ratio_in_basis_points: self
                .liquidity_pool
                .get_account_shares_ratio_in_basis_points(&account_id),
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

    // --- Liquidity Pool view methods ---

    /// Return liquidity pool configuration
    pub fn get_liquidity_pool_config(&self) -> LiquidityPoolConfig {
        self.liquidity_pool.config.clone()
    }

    // --- Staking Farm view methods ---

    /// Return all authorized users.
    // pub fn get_authorized_users(&self) -> Vec<AccountId> {
    //     self.authorized_users.to_vec()
    // }

    /// Return all authorized tokens.
    pub fn get_authorized_farm_tokens(&self) -> Vec<AccountId> {
        self.authorized_farm_tokens.to_vec()
    }

    pub fn get_active_farms(&self) -> Vec<HumanReadableFarm> {
        self.active_farms
            .iter()
            .map(|&index| HumanReadableFarm::from(index, self.farms.get(index).unwrap()))
            .collect()
    }

    pub fn get_farms(&self, from_index: u64, limit: u64) -> Vec<HumanReadableFarm> {
        (from_index..std::cmp::min(from_index + limit, self.farms.len()))
            .map(|index| HumanReadableFarm::from(index, self.farms.get(index).unwrap()))
            .collect()
    }

    pub fn get_farm(&self, farm_id: u64) -> HumanReadableFarm {
        HumanReadableFarm::from(farm_id, self.internal_get_farm(farm_id))
    }

    /// Get unclaimed rewards by account and farm
    pub fn get_unclaimed_reward(&self, account_id: AccountId, farm_id: u64) -> U128 {
        // if account_id == AccountId::new_unchecked(ZERO_ADDRESS.to_string()) {
        //     return U128(0);
        // }
        let account = self.accounts.get(&account_id).expect(ERR_NO_ACCOUNT);
        let mut farm = self.farms.get(farm_id).expect(ERR_NO_FARM);
        let (_rps, reward) = self.internal_unclaimed_balance(&account, farm_id, &mut farm);
        let prev_reward = *account.amounts.get(&farm.token_id).unwrap_or(&0);
        U128(reward + prev_reward)
    }
}
