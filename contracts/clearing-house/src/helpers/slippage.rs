use crate::error::ContractError;

use crate::helpers::casting::cast_to_i128;
use crate::helpers::constants::{
    MARK_PRICE_TIMES_AMM_TO_QUOTE_PRECISION_RATIO, PRICE_SPREAD_PRECISION,
};

pub fn calculate_slippage(
    exit_value: u128,
    base_asset_amount: u128,
    mark_price_before: i128,
) -> Result<i128, ContractError> {
    let amm_exit_price = exit_value
        .checked_mul(MARK_PRICE_TIMES_AMM_TO_QUOTE_PRECISION_RATIO)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    cast_to_i128(amm_exit_price)?
        .checked_sub(mark_price_before)
        .ok_or_else(|| (ContractError::MathError))
}

pub fn calculate_slippage_pct(
    slippage: i128,
    mark_price_before: i128,
) -> Result<i128, ContractError> {
    slippage
        .checked_mul(PRICE_SPREAD_PRECISION)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(mark_price_before)
        .ok_or_else(|| (ContractError::MathError))
}
