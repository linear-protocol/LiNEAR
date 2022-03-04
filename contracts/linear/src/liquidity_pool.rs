use crate::*;
use near_sdk::{
    near_bindgen, Balance, log, Promise,
    collections::LookupMap
};

// Mocked NEAR and LINEAR token used in Liquidity Pool
const NEAR_TOKEN_ACCOUNT: &str = "near";
const LINEAR_TOKEN_ACCOUNT: &str = "linear";


#[derive(BorshSerialize, BorshDeserialize)]
pub struct LiquidityPool {
    /// List of tokens in the pool
    pub token_account_ids: Vec<AccountId>,
    /// How much token in the pool
    pub amounts: Vec<Balance>,
    /// Shares of the pool by liquidity providers.
    pub shares: LookupMap<AccountId, Balance>,
    /// Total number of shares
    pub shares_total_supply: Balance,

    /// The amount of expected near amount to keep fee lower
    pub expected_near_amount: Balance,
    /// Max fee percentage
    pub max_fee: u32,
    /// Min fee percentage
    pub min_fee: u32,
    /// Fee allocated to DAO 
    pub fee_treasury_percentage: u32,
    /// Total swap fee in LiNEAR received by the pool
    pub total_fee_shares: ShareBalance,
}

pub struct Context {
    pub total_staked_near_amount: Balance,
    pub total_share_amount: ShareBalance
}

impl LiquidityPool {
    pub fn new(
        expected_near_amount: Balance,
        max_fee: u32,
        min_fee: u32,
        fee_treasury_percentage: u32,
    ) -> Self {
        require!(min_fee > 0, ERR_NON_POSITIVE_MIN_FEE);
        require!(max_fee >= min_fee, ERR_FEE_MAX_LESS_THAN_MIN);

        // Default token IDs
        let token_account_ids: Vec<AccountId> = Vec::from([
            NEAR_TOKEN_ACCOUNT.parse::<AccountId>().unwrap().clone(),
            LINEAR_TOKEN_ACCOUNT.parse::<AccountId>().unwrap().clone()
        ]);

        Self {
            token_account_ids: token_account_ids.clone(),
            amounts: vec![0u128; token_account_ids.len()],
            shares: LookupMap::new(StorageKey::Shares),
            shares_total_supply: 0,
            expected_near_amount,
            max_fee,
            min_fee,
            fee_treasury_percentage,
            total_fee_shares: 0,
        }
    }

    /// Adds the amounts of tokens to liquidity pool and returns number of shares that this user receives.
    pub fn add_liquidity(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        shares: Balance
    ) {
        require!(shares > 0, ERR_NON_POSITIVE_LIQUIDITY_POOL_SHARE);
        self.mint_shares(&account_id, shares);
        // Add NEAR amount
        self.amounts[0] += amount;
        log!(
            "Liquidity added {} NEAR, minted {} shares",
            amount,
            shares
        );
    }

    /// Removes given number of shares from the pool and returns amounts to the parent.
    pub fn remove_liquidity(
        &mut self,
        account_id: &AccountId,
        shares: Balance
    ) -> Vec<Balance> {
        let prev_shares_amount = self.shares.get(&account_id).expect(ERR_ACCOUNT_NO_SHARE);
        require!(
            prev_shares_amount >= shares,
            ERR_NO_ENOUGH_LIQUIDITY_SHARES_TO_REMOVE
        );

        let mut result = vec![];
        for i in 0..self.token_account_ids.len() {
            let amount = (U256::from(self.amounts[i]) * U256::from(shares)
                / U256::from(self.shares_total_supply))
            .as_u128();
            // require!(amount >= min_amounts[i], "ERR_MIN_AMOUNT");
            self.amounts[i] -= amount;
            result.push(amount);
        }
        if prev_shares_amount == shares {
            self.shares.insert(&account_id, &0);
        } else {
            self.shares
                .insert(&account_id, &(prev_shares_amount - shares));
        }
        log!(
            "{} shares of liquidity removed: receive back {:?}",
            shares,
            result
                .iter()
                .zip(self.token_account_ids.iter())
                .map(|(amount, token_id)| format!("{} {}", amount, token_id))
                .collect::<Vec<String>>(),
        );
        self.shares_total_supply -= shares;
        result
    }

    /// Swap NEAR token into LiNEAR and calculate the fees.
    pub fn swap(
        &mut self,
        requested_amount: Balance,      // NEAR
        stake_shares_in: ShareBalance,  // LiNEAR
        min_amount_out: Balance,
        context: &Context
    ) -> (Balance, ShareBalance) {
        // Calculate the swap fee percentage from requested amount
        let swap_fee_percentage = self.get_current_swap_fee_percentage(requested_amount);
        require!(swap_fee_percentage < ONE_HUNDRED_PERCENT, ERR_FEE_EXCEEDS_UP_LIMIT);
        // Calculate swap fee and received NEAR amount
        let swap_fee = (U256::from(requested_amount)
            * U256::from(swap_fee_percentage)
            / U256::from(ONE_HUNDRED_PERCENT)).as_u128();
        let received_amount = requested_amount - swap_fee;
        require!(self.amounts[0] >= received_amount, ERR_NO_ENOUGH_LIQUIDITY);
        require!(received_amount >= min_amount_out,
            format!(
                "The received NEAR {} will be less than the expected amount {}",
                received_amount,
                min_amount_out
            )
        );

        // Calculate LiNEAR amount for the swap fee
        let fee_num_shares = self.num_shares_from_staked_amount_rounded_down(
            swap_fee,
            context
        );
        let treasury_fee_shares = (U256::from(fee_num_shares)
            * U256::from(self.fee_treasury_percentage)
            / U256::from(ONE_HUNDRED_PERCENT)).as_u128();
        // Calculate the total received fee in LiNEAR
        let pool_fee_shares = fee_num_shares - treasury_fee_shares;
        require!(pool_fee_shares > 0, ERR_NON_POSITIVE_RECEIVED_FEE);
        self.total_fee_shares += pool_fee_shares;

        // Swap NEAR out of pool
        self.amounts[0] -= received_amount;

        // Swap LiNEAR into pool, excluding the fees for treasury
        let received_num_shares = stake_shares_in - treasury_fee_shares;
        self.amounts[1] += received_num_shares;

        (received_amount, treasury_fee_shares)
    }

    /// Rebalance pool distribution, increase NEAR and decrease LiNEAR
    pub fn rebalance(&mut self,
        requested_amount: Balance,
        context: &Context
    ) -> (Balance, ShareBalance) {
        let staked_shares = self.amounts[1];
        // If no requested amounts or no LiNEAR available, don't rebalance
        if requested_amount <= 0 || staked_shares <= 0 {
            return (0, 0);
        }
        // Calculate increased NEAR amount, and decreased LiNEAR amount
        let staked_shares_value = self.staked_amount_from_num_shares_rounded_down(
            staked_shares,
            &context
        );
        let (increased_amount, decreased_stake_shares) = if requested_amount >= staked_shares_value {
            (
                staked_shares_value,
                staked_shares
            )
        } else {
            (
                requested_amount,
                self.num_shares_from_staked_amount_rounded_down(requested_amount, &context),
            )
        };
        // Increase NEAR
        self.amounts[0] += increased_amount;
        // Decrease LiNEAR
        self.amounts[1] -= decreased_stake_shares;

        log!(
            "Liquidity has been rebalanced by adding {} NEAR and removing {} LiNEAR",
            increased_amount,
            decreased_stake_shares
        );

        (increased_amount, decreased_stake_shares)
    }

    /// Calculate NEAR value from shares, rounding down
    pub fn get_value_from_shares_rounded_down(
        &self,
        shares: Balance,
        context: &Context
    ) -> Balance {
        if self.shares_total_supply == 0 || shares == 0 {
            0
        } else {
            let pool_value_in_near = self.get_pool_value(context);
            (U256::from(pool_value_in_near) * U256::from(shares)
                / U256::from(self.shares_total_supply))
            .as_u128()
        }
    }

    /// Calculate NEAR value from shares, rounding up
    pub fn get_value_from_shares_rounded_up(
        &self,
        shares: Balance,
        context: &Context
    ) -> Balance {
        if self.shares_total_supply == 0 || shares == 0 {
            0
        } else {
            let pool_value_in_near = self.get_pool_value(context);
            ((U256::from(pool_value_in_near) * U256::from(shares)
                + U256::from(self.shares_total_supply - 1))
                / U256::from(self.shares_total_supply))
            .as_u128()
        }
    }

    /// Calculate shares from give value in NEAR, rounding down
    pub fn get_shares_from_value_rounded_down(
        &self,
        amount: Balance,
        context: &Context
    ) -> Balance {
        let pool_value_in_near = self.get_pool_value(context);
        if self.shares_total_supply == 0 {
            amount
        } else if amount == 0 || pool_value_in_near == 0 {
            0
        } else {
            (U256::from(amount) * U256::from(self.shares_total_supply)
                / U256::from(pool_value_in_near)).as_u128()
        }
    }

    /// Calculate shares from give value in NEAR, rounding up
    pub fn get_shares_from_value_rounded_up(
        &self,
        amount: Balance,
        context: &Context
    ) -> Balance {
        let pool_value_in_near = self.get_pool_value(context);
        if self.shares_total_supply == 0 {
            amount
        } else if amount == 0 || pool_value_in_near == 0 {
            0
        } else {
            ((U256::from(amount) * U256::from(self.shares_total_supply)
                + U256::from(pool_value_in_near - 1))
                / U256::from(pool_value_in_near))
            .as_u128()
        }
    }

    /// Calculate the Liquidity Pool value in NEAR
    fn get_pool_value(
        &self,
        context: &Context
    ) -> Balance {
        self.amounts[0] +
            self.staked_amount_from_num_shares_rounded_down(
                self.amounts[1],
                context
            )
    }

    /// Return shares for the account
    pub fn get_account_shares(&self, account_id: &AccountId) -> ShareBalance {
        self.shares.get(&account_id).unwrap_or(0)
    }

    /// Calculate account value in NEAR by shares
    pub fn get_account_value(
        &self,
        account_id: &AccountId,
        context: &Context
    ) -> Balance {
        let shares = self.get_account_shares(&account_id);
        self.get_value_from_shares_rounded_up(shares, context)
    }

    /// Calculate account liquidity pool shares percentage
    pub fn get_account_shares_percentage(&self, account_id: &AccountId) -> u32 {
        let shares = self.get_account_shares(&account_id);
        if self.shares_total_supply == 0 || shares == 0 {
            0
        } else {
            (U256::from(ONE_HUNDRED_PERCENT)
                * U256::from(shares)
                / U256::from(self.shares_total_supply)).as_u32()
        }
    }

    /// Mint new shares for given user.
    fn mint_shares(&mut self, account_id: &AccountId, shares: Balance) {
        if shares == 0 {
            return;
        }
        let prev_shares_amount = self.get_account_shares(&account_id);
        self.shares.insert(&account_id, &(prev_shares_amount + shares));
        self.shares_total_supply += shares;
    }

    fn num_shares_from_staked_amount_rounded_down(
        &self,
        amount: Balance,
        context: &Context
    ) -> ShareBalance {
        require!(context.total_staked_near_amount > 0, ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE);
        (U256::from(context.total_share_amount) * U256::from(amount)
            / U256::from(context.total_staked_near_amount))
        .as_u128()
    }

    fn staked_amount_from_num_shares_rounded_down(
        &self,
        num_shares: ShareBalance,
        context: &Context
    ) -> Balance {
        require!(context.total_share_amount > 0, ERR_NON_POSITIVE_TOTAL_STAKE_SHARES);
        (U256::from(context.total_staked_near_amount) * U256::from(num_shares)
            / U256::from(context.total_share_amount))
        .as_u128()
    }

    /// Swap fee calculated based on swap amount
    pub fn get_current_swap_fee_percentage(&self, amount_out: u128) -> u32 {
        if self.amounts[0] <= amount_out {
            return self.max_fee;
        }

        let remaining_amount = self.amounts[0] - amount_out;
        if remaining_amount >= self.expected_near_amount {
            return self.min_fee;
        }

        let diff = self.max_fee - self.min_fee;
        self.max_fee -
            (U256::from(diff) * U256::from(remaining_amount) 
                / U256::from(self.expected_near_amount))
                .as_u32()
    }

}

/// The single-direction liquidity pool that enables swapping LiNEAR 
/// into NEAR instantly
#[near_bindgen]
impl LiquidStakingContract {
    /// Adds NEAR to liquidity pool and returns number of shares that this user receives.
    #[payable]
    pub fn add_liquidity(&mut self) {
        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();

        // Calculate liquidity pool shares, rounding down
        let added_shares = self.liquidity_pool.get_shares_from_value_rounded_down(
            amount,
            &self.internal_get_context()
        );
        // Add shares in liquidity pool
        self.liquidity_pool.add_liquidity(
            &account_id,
            amount,
            added_shares
        );

        // Update the toal balance
        // TODO: fix the usage of last_total_balance
        self.last_total_balance += amount;
    }

    /// Remove shares from the liquidity pool and return NEAR and LiNEAR.
    /// The parameter `amount` means the value of NEAR to be removed
    pub fn remove_liquidity(&mut self, amount: U128) -> Vec<U128> {
        let account_id = env::predecessor_account_id();
        let amount: Balance = amount.into();

        // Is this necessary? already asserted by shares in next steps
        // Calculate the NEAR value owned by the account
        // let account_value = self.liquidity_pool.get_account_value(
        //     &account_id,
        //     self.internal_get_context()
        // );
        // require!(
        //     account_value >= amount,
        //     ERR_NO_ENOUGH_LIQUIDITY_SHARES_TO_REMOVE
        // );

        // Calculate liquidity pool shares from NEAR amount
        let mut removed_shares = self.liquidity_pool.get_shares_from_value_rounded_up(
            amount,
            &self.internal_get_context()
        );
        // Remove shares from liquidity pool
        let results = self.liquidity_pool.remove_liquidity(
            &account_id,
            removed_shares
        );

        // Receive NEAR and LiNEAR
        let mut account = self.internal_get_account(&account_id);
        account.stake_shares += results[1];
        self.internal_save_account(&account_id, &account);
        Promise::new(env::predecessor_account_id()).transfer(results[0]);

        results.iter()
            .map(|amount| amount.clone().into())
            .collect()
    }

    /// Instant Unstake: swap LiNEAR to NEAR via the Liquidity Pool
    /// Notice that total staked NEAR amount and total staked shares won't change here
    pub fn instant_unstake(
        &mut self,
        staked_shares_in: U128,     // LiNEAR amount sent by the account
        min_amount_out: U128        // Minimal NEAR amount should be returned
    ) -> U128 {
        let staked_shares_in: ShareBalance = staked_shares_in.into();
        require!(staked_shares_in > 0, ERR_NON_POSITIVE_UNSTAKING_AMOUNT);
        let min_amount_out: Balance = min_amount_out.into();
        require!(min_amount_out > 0, ERR_NON_POSITIVE_MIN_RECEIVED_AMOUNT);

        let account_id = env::predecessor_account_id();
        let mut account = self.internal_get_account(&account_id);
        require!(account.stake_shares >= staked_shares_in, ERR_NO_ENOUGH_STAKED_BALANCE);

        // Distribute rewards from all the farms for the given user.
        self.internal_distribute_all_rewards(&mut account);

        // Calculating the amount of tokens the account will receive by unstaking the corresponding
        // number of "stake" shares, rounding up.
        let requested_amount = self.staked_amount_from_num_shares_rounded_up(staked_shares_in);
        require!(requested_amount > 0, ERR_NON_POSITIVE_CALCULATED_STAKED_AMOUNT);

        // Swap NEAR out from liquidity pool
        let (received_amount, treasury_fee_shares) = self.liquidity_pool.swap(
            requested_amount,
            staked_shares_in,
            min_amount_out,
            &self.internal_get_context()
        );

        // Calculate and distribute fees for DAO treasury
        let treasury_account_id = TREASURY_ACCOUNT.parse::<AccountId>().unwrap();
        let mut treasury_account = self.internal_get_account(&treasury_account_id);
        treasury_account.stake_shares += treasury_fee_shares;
        self.internal_save_account(&treasury_account_id, &treasury_account);

        // Update account staked shares
        account.stake_shares -= staked_shares_in;
        self.internal_save_account(&account_id, &account);
        // Transfer NEAR to account
        Promise::new(account_id.clone()).transfer(received_amount);

        log!(
            "@{} instantly unstaked {} LiNEAR, received {} NEAR",
            &account_id,
            staked_shares_in,
            received_amount
        );

        received_amount.into()
    }

    /// Provide context that are useful in modules
    pub(crate) fn internal_get_context(&self) -> Context {
        Context {
            total_staked_near_amount: self.total_staked_near_amount,
            total_share_amount: self.total_share_amount
        }
    }

    /// Rebalance NEAR / LiNEAR distribution to make the liqudity pool more efficient
    /// Automatically swap LiNEAR out with newly staked NEAR
    pub(crate) fn rebalance_liquidity(&mut self) {
        // If no new staking request, skip the rebalance
        if self.epoch_requested_stake_amount <= 0 {
            return;
        }
        // Rebalance in the pool and return actual rebalanced amount and staked shares
        let (increased_amount, decreased_staked_shares) = self.liquidity_pool.rebalance(
            self.epoch_requested_stake_amount,
            &self.internal_get_context()
        );
        // Reverse the staking request, to mitigate the side effect of instant unstake
        // Decrease staked amount, which now has been moved into liquidity pool
        self.epoch_requested_stake_amount -= increased_amount;
        self.total_staked_near_amount -= increased_amount;
        // Decrease staked shares
        self.total_share_amount -= decreased_staked_shares;
    }
}
