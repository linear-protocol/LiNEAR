use crate::*;
use near_sdk::{
    near_bindgen, AccountId,
    json_types::{U128, U64}
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
    /// For example, fee percentage is `30`, which means `0.3%`
    pub lp_swap_fee_percentage: u32,

    /// Active farms that affect stakers.
    /// Can calculate rate of return of this pool with farming by:
    /// `farm_reward_per_day = farms.iter().map(farms.amount / (farm.end_date - farm.start_date) / DAY_IN_NS * PRICES[farm.token_id]).sum()`
    /// `near_reward_per_day = total_near_emission_per_day * this.total_staked_near_amount / total_near_staked`
    /// `total_reward_per_day = farm_reward_per_day + near_reward_per_day * NEAR_PRICE`
    /// `reward_rate = total_reward_per_day / (this.total_staked_near_amount * NEAR_PRICE)`
    pub farms: Vec<HumanReadableFarm>,
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
    pub fn get_total_share_amount(& self) -> ShareBalance {
        self.total_share_amount
    }

    pub fn get_total_staked_near_amount(& self) -> Balance {
        self.total_staked_near_amount
    }

    pub fn get_beneficiaries(& self) -> HashMap<AccountId, Fraction> {
        self.internal_get_beneficiaries()
    }
  
    pub fn get_summary(&self) -> Summary {
      Summary {
          total_share_amount: self.total_share_amount.into(),
          total_staked_near_amount: self.total_staked_near_amount.into(),
          ft_price: self.ft_price(),
          lp_target_amount: self.liquidity_pool.expected_near_amount.into(),
          lp_near_amount: self.liquidity_pool.amounts[0].into(),
          lp_staked_share: self.liquidity_pool.amounts[1].into(),
          lp_swap_fee_percentage: self.liquidity_pool.get_current_swap_fee_percentage(10 * ONE_NEAR),
          farms: self.get_active_farms(),
      }
    }
  
    // Staking Farm

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
        let account = self.accounts.get(&account_id).expect("ERR_NO_ACCOUNT");
        let mut farm = self.farms.get(farm_id).expect("ERR_NO_FARM");
        let (_rps, reward) = self.internal_unclaimed_balance(&account, farm_id, &mut farm);
        let prev_reward = *account.amounts.get(&farm.token_id).unwrap_or(&0);
        U128(reward + prev_reward)
    }
}
