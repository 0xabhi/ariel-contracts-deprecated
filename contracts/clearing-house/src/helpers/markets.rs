use crate::error::ContractError;

use crate::states::market::{Amm};

use crate::helpers::amm;

pub fn get_mark_price(a: &Amm) -> Result<u128, ContractError> {
    amm::calculate_price(
        a.quote_asset_reserve,
        a.base_asset_reserve,
        a.peg_multiplier,
    )

}
