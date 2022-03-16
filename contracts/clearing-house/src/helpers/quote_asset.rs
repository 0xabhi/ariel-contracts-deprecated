use crate::error::ContractError;

use crate::helpers::constants::AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO;

pub fn reserve_to_asset_amount(
    quote_asset_reserve: u128,
    peg_multiplier: u128,
) -> Result<u128, ContractError> {
    Ok(quote_asset_reserve
        .checked_mul(peg_multiplier).ok_or_else(|| (ContractError::MathError))?
        .checked_div(AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO).ok_or_else(|| (ContractError::MathError))?
    )
}

pub fn asset_to_reserve_amount(
    quote_asset_amount: u128,
    peg_multiplier: u128,
) -> Result<u128, ContractError> {
    Ok(quote_asset_amount
        .checked_mul(AMM_TIMES_PEG_TO_QUOTE_PRECISION_RATIO).ok_or_else(|| (ContractError::MathError))?
        .checked_div(peg_multiplier).ok_or_else(|| (ContractError::MathError))?
    )
}
