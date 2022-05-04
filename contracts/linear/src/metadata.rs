use crate::*;
use near_sdk::near_bindgen;

/// To make it easier for the contract to be audited and validated by community
/// and 3rd party, we adopt [NEP-330 standard](https://github.com/near/NEPs/blob/master/neps/nep-0330.md)
/// to make contract source metadata (including versions and source code links)
/// available to auditors, developers and users.
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractSourceMetadata {
    pub version: String,
    pub link: String,
}

pub trait ContractSourceMetadataTrait {
    fn contract_source_metadata(&self) -> ContractSourceMetadata;
}

#[near_bindgen]
impl ContractSourceMetadataTrait for LiquidStakingContract {
    fn contract_source_metadata(&self) -> ContractSourceMetadata {
        ContractSourceMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            link: "https://github.com/linear-protocol/LiNEAR".to_string(),
        }
    }
}
