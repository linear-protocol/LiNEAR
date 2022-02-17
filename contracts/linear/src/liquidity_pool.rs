use crate::*;
use near_sdk::{
    near_bindgen, Balance, log, Promise,
    collections::LookupMap
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct LiquidityPool {
    /// How much NEAR in the pool
    pub near_amount: Balance,
    /// How much LiNEAR in the pool
    pub linear_amount: Balance,
    /// Shares of the pool by liquidity providers.
    pub shares: LookupMap<AccountId, Balance>,
    /// Total number of shares.
    pub shares_total_supply: Balance,

    /// The amount of near amount required to keep fee lower
    pub required_near_amount: Balance,
    /// Max fee percentage
    pub max_fee: u32,
    /// Min fee percentage
    pub min_fee: u32,
    /// Fee allocated to DAO 
    pub fee_treasury_percentage: u32,
}

impl LiquidityPool {
    pub fn new(
        required_near_amount: Balance,
        max_fee: u32,
        min_fee: u32,
        fee_treasury_percentage: u32,
    ) -> Self {
        require!(min_fee > 0, ERR_NON_POSITIVE_MIN_FEE);
        require!(max_fee >= min_fee, ERR_FEE_MAX_LESS_THAN_MIN);

        Self {
            near_amount: 0,
            linear_amount: 0,
            required_near_amount,
            max_fee,
            min_fee,
            shares: LookupMap::new(StorageKey::Shares),
            shares_total_supply: 0,
            fee_treasury_percentage,
        }
    }

    pub fn swap(
        &mut self,
        requested_amount: Balance,
        num_shares_in: ShareBalance,
        min_amount_out: Balance,
        total_staked_near_amount: Balance,
        total_share_amount: ShareBalance,
    ) -> u128 {
        // Calculating the swap fee percentage from the receive_amount
        let swap_fee_percentage = self.get_current_swap_fee(requested_amount);
        require!(swap_fee_percentage < FULL_PERCENTAGE, ERR_FEE_EXCEEDS_UP_LIMIT);
        let swap_fee = (U256::from(requested_amount) * U256::from(swap_fee_percentage) 
            / U256::from(FULL_PERCENTAGE)).as_u128();
        let received_amount = requested_amount - swap_fee;
        require!(received_amount > self.near_amount, ERR_NO_ENOUGH_LIQUIDITY);
        require!(received_amount > min_amount_out, 
            format!("The received NEAR {} will be less than the expected amount {}", 
                received_amount, min_amount_out)
        );

        // Swap out NEAR from pool
        self.near_amount -= received_amount;

        // Calculate LiNEAR amount for the swap_fee
        let fee_num_shares = self.num_shares_from_staked_amount_rounded_down(
            swap_fee,
            total_staked_near_amount,
            total_share_amount
        );
        let treasury_fee = (U256::from(fee_num_shares) * U256::from(self.fee_treasury_percentage) 
            / U256::from(FULL_PERCENTAGE)).as_u128();

        // Swap in LiNEAR into pool, excluding the fees for treasury
        let received_num_shares = num_shares_in - treasury_fee;
        self.linear_amount += received_num_shares;

        treasury_fee
    }

    /// Swap fee calculated based on swap amount
    pub fn get_current_swap_fee(&self, amount_out: u128) -> u32 {
        if self.near_amount <= amount_out {
            return self.max_fee;
        }

        let remaining_amount = self.near_amount - amount_out;
        if remaining_amount >= self.required_near_amount {
            return self.min_fee;
        }

        let diff = self.max_fee - self.min_fee;
        return self.max_fee - 
            (U256::from(diff) * U256::from(remaining_amount) 
                / U256::from(self.required_near_amount))
                .as_u32();
    }

    pub(crate) fn num_shares_from_staked_amount_rounded_down(
        &self,
        amount: Balance,
        total_staked_near_amount: Balance,
        total_share_amount: ShareBalance,
    ) -> ShareBalance {
        require!(total_staked_near_amount > 0, ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE);
        (U256::from(total_share_amount) * U256::from(amount)
            / U256::from(total_staked_near_amount))
        .as_u128()
    }
}

/// The single-direction liquidity pool that enables swapping LiNEAR 
/// into NEAR instantly
#[near_bindgen]
impl LiquidStakingContract {
    #[payable]
    pub fn add_liquidity(&mut self) {
     
    }

    pub fn remove_liquidity(&mut self) {
        
    }

    /// Instant Unstake: swap LiNEAR to NEAR via the Liquidity Pool
    pub fn instant_unstake(&mut self, amount_in: U128, min_amount_out: U128) {
        let amount_in: ShareBalance = amount_in.into();
        require!(amount_in > 0, ERR_NON_POSITIVE_UNSTAKING_AMOUNT);
        let min_amount_out: Balance = min_amount_out.into();
        require!(min_amount_out > 0, ERR_NON_POSITIVE_MIN_RECEIVED_AMOUNT);

        require!(self.total_staked_near_amount > 0, ERR_CONTRACT_NO_STAKED_BALANCE);

        let account_id = env::predecessor_account_id();
        let mut account = self.internal_get_account(&account_id);
        require!(account.stake_shares >= amount_in, ERR_NO_ENOUGH_STAKED_BALANCE);

        // Calculating the amount of tokens the account will receive by unstaking the corresponding
        // number of "stake" shares, rounding up.
        let num_shares = amount_in;
        let receive_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);
        require!(receive_amount > 0, ERR_NON_POSITIVE_CALCULATED_STAKED_AMOUNT);

        // Swap NEAR out from liquidity pool
        let treasury_fee = self.liquidity_pool.swap(receive_amount, num_shares, min_amount_out,
            self.total_staked_near_amount, self.total_share_amount
        );

        // Calculate and distribute fees for DAO treasury
        let treasury_account_id = "treasury".parse::<AccountId>().unwrap();
        let mut treasury_account = self.internal_get_account(&treasury_account_id);
        treasury_account.stake_shares += treasury_fee;
        self.internal_save_account(&treasury_account_id, &treasury_account);

        // Update account balance and shares
        account.stake_shares -= num_shares;
        account.unstaked += receive_amount;
        self.internal_save_account(&account_id, &account);
        Promise::new(env::predecessor_account_id()).transfer(receive_amount);

        log!(
            "@{} instantly unstaked {} LiNEAR, received {} NEAR",
            &account_id,
            amount_in,
            receive_amount
        );

        // The amount tokens that will be unstaked from the total to guarantee the "stake" share
        // price never decreases. The difference between `receive_amount` and `unstake_amount` is
        // paid from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        // let unstake_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);

        // self.total_staked_near_amount -= unstake_amount;
        // self.total_share_amount -= num_shares;
    }

}