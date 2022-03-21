use crate::*;
use near_sdk::json_types::U128;

pub trait FungibleTokenPrice {
    fn ft_price(&self) -> U128;
}

#[near_bindgen]
impl FungibleTokenPrice for LiquidStakingContract {
    fn ft_price(&self) -> U128 {
        let amount = self.staked_amount_from_num_shares_rounded_down(ONE_NEAR);
        amount.into()
    }
}
