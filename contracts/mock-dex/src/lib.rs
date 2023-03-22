use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    log, near_bindgen, require, AccountId, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct Contract {}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let amount = amount.into();
        require!(amount > 0, "received amount must be positive");
        log!("sender: {}, amount: {}, msg: {}", sender_id, amount, msg);
        log!(
            "predecessor: {}, attached gas: {}T, deposit: {}yoctoN",
            env::predecessor_account_id(),
            u64::from(env::prepaid_gas()) as f64 / 1e12,
            env::attached_deposit()
        );
        match msg.as_str() {
            "fail" => {
                env::panic_str(format!("ft_on_transfer() from @{sender_id} failed!").as_str())
            }
            "refund" => PromiseOrValue::Value(U128(amount)),
            _ => PromiseOrValue::Value(U128(0)),
        }
    }
}
