use cosmwasm_std::Uint128;

use crate::error::ContractError;

use crate::helpers::constants::AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO;

pub fn reserve_to_asset_amount(
    quote_asset_reserve: Uint128,
    peg_multiplier: Uint128,
) -> Result<Uint128, ContractError> {
    Ok(quote_asset_reserve
        .checked_mul(peg_multiplier)?
        .checked_div(AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO)?
    )
}

pub fn asset_to_reserve_amount(
    quote_asset_amount: Uint128,
    peg_multiplier: Uint128,
) -> Result<Uint128, ContractError> {
    Ok(quote_asset_amount
        .checked_mul(AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO)?
        .checked_div(peg_multiplier)?
    )
}
