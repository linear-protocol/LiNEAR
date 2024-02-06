//! The built-in liquidity pool feature has been deprecated.
//! Keep the legacy structs for tracking contract states.

use crate::*;
use near_sdk::{collections::LookupMap, Balance};

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

    /// Configuration of the pool
    pub config: LiquidityPoolConfig,

    /// Total swap fee in LiNEAR received by the pool
    pub total_fee_shares: ShareBalance,
}

impl LiquidityPool {
    pub fn new(config: LiquidityPoolConfig) -> Self {
        // Default token IDs
        let token_account_ids: Vec<AccountId> = Vec::from([
            NEAR_TOKEN_ACCOUNT.parse::<AccountId>().unwrap(),
            LINEAR_TOKEN_ACCOUNT.parse::<AccountId>().unwrap(),
        ]);

        Self {
            token_account_ids: token_account_ids.clone(),
            amounts: vec![0u128; token_account_ids.len()],
            shares: LookupMap::new(StorageKey::Shares),
            shares_total_supply: 0,
            config,
            total_fee_shares: 0,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidityPoolConfig {
    /// The expected near amount used in the fee calculation formula.
    /// If the NEAR amount in the liquidity pool exceeds the expectation, the
    /// swap fee will be the `min_fee_bps`
    pub expected_near_amount: U128,
    /// Max fee in basis points
    pub max_fee_bps: u32,
    /// Min fee in basis points
    pub min_fee_bps: u32,
    /// Fee allocated to treasury in basis points
    pub treasury_fee_bps: u32,
}

impl Default for LiquidityPoolConfig {
    fn default() -> Self {
        Self {
            expected_near_amount: U128(10000 * ONE_NEAR),
            max_fee_bps: 300,
            min_fee_bps: 30,
            treasury_fee_bps: 3000,
        }
    }
}
