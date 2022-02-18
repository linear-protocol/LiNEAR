use crate::*;
use near_sdk::{
    near_bindgen, Balance, log, Promise,
    collections::LookupMap
};

// NEAR and LINEAR token used in Liquidity Pool
const NEAR_TOKEN_ACCOUNT: &str = "n.linear";
const LINEAR_TOKEN_ACCOUNT: &str = "l.linear";


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

    /// The amount of near amount required to keep fee lower
    pub required_near_amount: Balance,
    /// Max fee percentage
    pub max_fee: u32,
    /// Min fee percentage
    pub min_fee: u32,
    /// Fee allocated to DAO 
    pub fee_treasury_percentage: u32,
}

pub struct Context {
    pub total_staked_near_amount: Balance,
    pub total_share_amount: ShareBalance
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

        // Default token IDs
        let token_account_ids: Vec<AccountId> = Vec::from([
            NEAR_TOKEN_ACCOUNT.parse::<AccountId>().unwrap().clone(),
            LINEAR_TOKEN_ACCOUNT.parse::<AccountId>().unwrap().clone()
        ]);

        Self {
            token_account_ids: token_account_ids.clone(),
            amounts: vec![0u128; token_account_ids.len()],
            required_near_amount,
            max_fee,
            min_fee,
            shares: LookupMap::new(StorageKey::Shares),
            shares_total_supply: 0,
            fee_treasury_percentage,
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
        let prev_shares_amount = self.get_account_shares(&account_id);
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
        context: Context
    ) -> u128 {
        // Calculating the swap fee percentage from requested amount
        let swap_fee_percentage = self.get_current_swap_fee_percentage(requested_amount);
        require!(swap_fee_percentage < ONE_HUNDRED_PERCENT, ERR_FEE_EXCEEDS_UP_LIMIT);
        let swap_fee = (U256::from(requested_amount) * U256::from(swap_fee_percentage) 
            / U256::from(ONE_HUNDRED_PERCENT)).as_u128();
        let received_amount = requested_amount - swap_fee;
        require!(self.amounts[0] > received_amount, ERR_NO_ENOUGH_LIQUIDITY);
        require!(received_amount > min_amount_out,
            format!(
                "The received NEAR {} will be less than the expected amount {}",
                received_amount,
                min_amount_out
            )
        );

        // Swap out NEAR from pool
        self.amounts[0] -= received_amount;

        // Calculate LiNEAR amount for the swap_fee
        let fee_num_shares = self.num_shares_from_staked_amount_rounded_down(
            swap_fee,
            context
        );
        let treasury_fee = (U256::from(fee_num_shares) * U256::from(self.fee_treasury_percentage) 
            / U256::from(ONE_HUNDRED_PERCENT)).as_u128();

        // Swap in LiNEAR into pool, excluding the fees for treasury
        let received_num_shares = stake_shares_in - treasury_fee;
        self.amounts[1] += received_num_shares;

        treasury_fee
    }

    /// Calculate account value in NEAR by shares
    pub fn get_account_value(
        &self,
        account_id: &AccountId,
        context: Context
    ) -> Balance {
        let shares = self.get_account_shares(&account_id);
        self.get_value_from_shares(shares, context)
    }

    /// Calculate NEAR value from shares
    pub fn get_value_from_shares(
        &self,
        shares: Balance,
        context: Context
    ) -> Balance {
        let pool_value_in_near = self.get_pool_value(context);
        (U256::from(shares) * U256::from(pool_value_in_near)
            / U256::from(self.shares_total_supply))
        .as_u128()
    }

    /// Calculate shares from give value in NEAR
    pub fn get_shares_from_value(
        &self,
        amount: Balance,
        context: Context
    ) -> Balance {
        let pool_value_in_near = self.get_pool_value(context);
        (U256::from(amount) * U256::from(self.shares_total_supply)
            / U256::from(pool_value_in_near))
        .as_u128()
    }

    /// Calculate the Liquidity Pool value in NEAR
    fn get_pool_value(
        &self,
        context: Context
    ) -> Balance {
        self.amounts[0] +
            self.staked_amount_from_num_shares_rounded_down(
                self.shares_total_supply,
                context
            )
    }

    /// Return shares for the account
    fn get_account_shares(&self, account_id: &AccountId) -> ShareBalance {
        self.shares.get(&account_id).expect(ERR_ACCOUNT_NO_SHARE)
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
        context: Context
    ) -> ShareBalance {
        require!(context.total_staked_near_amount > 0, ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE);
        (U256::from(context.total_share_amount) * U256::from(amount)
            / U256::from(context.total_staked_near_amount))
        .as_u128()
    }

    fn staked_amount_from_num_shares_rounded_down(
        &self,
        num_shares: ShareBalance,
        context: Context
    ) -> Balance {
        require!(context.total_share_amount > 0, ERR_NON_POSITIVE_TOTAL_STAKE_SHARES);
        (U256::from(context.total_staked_near_amount) * U256::from(num_shares)
            / U256::from(context.total_share_amount))
        .as_u128()
    }

    /// Swap fee calculated based on swap amount
    fn get_current_swap_fee_percentage(&self, amount_out: u128) -> u32 {
        if self.amounts[0] <= amount_out {
            return self.max_fee;
        }

        let remaining_amount = self.amounts[0] - amount_out;
        if remaining_amount >= self.required_near_amount {
            return self.min_fee;
        }

        let diff = self.max_fee - self.min_fee;
        self.max_fee -
            (U256::from(diff) * U256::from(remaining_amount) 
                / U256::from(self.required_near_amount))
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
        // Deposit will update the toal balance
        self.internal_deposit();

        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();

        // Add shares in liquidity pool
        let added_shares = self.liquidity_pool.get_shares_from_value(
            amount,
            self.get_context()
        );
        self.liquidity_pool.add_liquidity(
            &account_id,
            amount,
            added_shares
        );

        // Update unstaked amount
        let mut account = self.internal_get_account(&account_id);
        account.unstaked -= amount;
        self.internal_save_account(&account_id, &account);
    }

    /// Remove shares from the liquidity pool and return NEAR and LiNEAR
    pub fn remove_liquidity(&mut self, amount: U128) -> Vec<Balance> {
        let account_id = env::predecessor_account_id();
        let amount: Balance = amount.into();

        // Is this necessary? already asserted by shares in next steps
        // Calculate the NEAR value owned by the account
        // let account_value = self.liquidity_pool.get_account_value(
        //     &account_id,
        //     self.get_context()
        // );
        // require!(
        //     account_value >= amount,
        //     ERR_NO_ENOUGH_LIQUIDITY_SHARES_TO_REMOVE
        // );

        // Remove shares from liqudity pool
        let removed_shares = self.liquidity_pool.get_shares_from_value(
            amount,
            self.get_context()
        );
        let results = self.liquidity_pool.remove_liquidity(
            &account_id,
            removed_shares
        );

        let mut account = self.internal_get_account(&account_id);
        account.stake_shares += results[1];
        self.internal_save_account(&account_id, &account);
        Promise::new(env::predecessor_account_id()).transfer(results[0]);

        results
    }

    /// Instant Unstake: swap LiNEAR to NEAR via the Liquidity Pool
    pub fn instant_unstake(
        &mut self,
        staked_shares_in: U128,     // LiNEAR
        min_amount_out: U128
    ) -> Balance {
        let staked_shares_in: ShareBalance = staked_shares_in.into();
        require!(staked_shares_in > 0, ERR_NON_POSITIVE_UNSTAKING_AMOUNT);
        let min_amount_out: Balance = min_amount_out.into();
        require!(min_amount_out > 0, ERR_NON_POSITIVE_MIN_RECEIVED_AMOUNT);

        require!(self.total_staked_near_amount > 0, ERR_CONTRACT_NO_STAKED_BALANCE);

        let account_id = env::predecessor_account_id();
        let mut account = self.internal_get_account(&account_id);
        require!(account.stake_shares >= staked_shares_in, ERR_NO_ENOUGH_STAKED_BALANCE);

        // Calculating the amount of tokens the account will receive by unstaking the corresponding
        // number of "stake" shares, rounding up.
        let num_shares = staked_shares_in;
        let received_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);
        require!(received_amount > 0, ERR_NON_POSITIVE_CALCULATED_STAKED_AMOUNT);

        // Swap NEAR out from liquidity pool
        let treasury_fee = self.liquidity_pool.swap(
            received_amount,
            num_shares,
            min_amount_out,
            self.get_context()
        );

        // Calculate and distribute fees for DAO treasury
        let treasury_account_id = TREASURY_ACCOUNT.parse::<AccountId>().unwrap();
        let mut treasury_account = self.internal_get_account(&treasury_account_id);
        treasury_account.stake_shares += treasury_fee;
        self.internal_save_account(&treasury_account_id, &treasury_account);

        // Update account balance and shares
        account.stake_shares -= num_shares;
        account.unstaked += received_amount;
        self.internal_save_account(&account_id, &account);
        Promise::new(env::predecessor_account_id()).transfer(received_amount);

        log!(
            "@{} instantly unstaked {} LiNEAR, received {} NEAR",
            &account_id,
            staked_shares_in,
            received_amount
        );

        received_amount
    }

    /// Provide context that are useful in modules
    fn get_context(&self) -> Context {
        Context {
            total_staked_near_amount: self.total_staked_near_amount,
            total_share_amount: self.total_share_amount
        }
    }

}