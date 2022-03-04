use crate::*;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg width='36' height='35' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M36 33V19l-1-1L6 1H4L1 2 0 5v12l1 1 29 16 2 1 2-1 2-1zm0-28V3l-2-2h-4l-9 5v1l14 8h1V5zM0 30l1 2 1 2 2 1 2-1 9-5v-1h-1L2 21v-1H0v10z' fill='url(%23paint0_linear_19_80)'/%3E%3Cdefs%3E%3ClinearGradient id='paint0_linear_19_80' x1='2.743' y1='1.96' x2='34.816' y2='34.956' gradientUnits='userSpaceOnUse'%3E%3Cstop stop-color='%231BB3CC'/%3E%3Cstop offset='1' stop-color='%23824ACC'/%3E%3C/linearGradient%3E%3C/defs%3E%3C/svg%3E";

#[near_bindgen]
impl FungibleTokenMetadataProvider for LiquidStakingContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: String::from("LiNEAR"),
            symbol: String::from("LINEAR"),
            icon: Some(String::from(DATA_IMAGE_SVG_NEAR_ICON)),
            reference: None,
            reference_hash: None,
            decimals: 24,
        }
    }
}
