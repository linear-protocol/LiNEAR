use crate::*;
use near_sdk::{
    near_bindgen, AccountId,
    json_types::{U128, U64}
};

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

/// Represents pool summary with all farms and rates applied.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolSummary {
    /// Pool owner.
    pub owner: AccountId,
    /// The total staked balance.
    pub total_staked_balance: Balance,
    /// Active farms that affect stakers.
    pub farms: Vec<HumanReadableFarm>,
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


    /// Return all authorized users.
    // pub fn get_authorized_users(&self) -> Vec<AccountId> {
    //     self.authorized_users.to_vec()
    // }

    /// Return all authorized tokens.
    pub fn get_authorized_farm_tokens(&self) -> Vec<AccountId> {
        self.authorized_farm_tokens.to_vec()
    }

    ///
    /// FARMS
    ///

    /// Returns summary of this pool.
    /// Can calculate rate of return of this pool with farming by:
    /// `farm_reward_per_day = farms.iter().map(farms.amount / (farm.end_date - farm.start_date) / DAY_IN_NS * PRICES[farm.token_id]).sum()`
    /// `near_reward_per_day = total_near_emission_per_day * this.total_staked_balance / total_near_staked`
    /// `total_reward_per_day = farm_reward_per_day + near_reward_per_day * NEAR_PRICE`
    /// `reward_rate = total_reward_per_day / (this.total_staked_balance * NEAR_PRICE)`
    pub fn get_pool_summary(&self) -> PoolSummary {
        PoolSummary {
            owner: self.get_owner_id(),
            total_staked_balance: self.total_share_amount,
            farms: self.get_active_farms(),
        }
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
