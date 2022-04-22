use cosmwasm_std::Uint128;

use crate::error::ContractError;

use crate::helpers::constants::{
    MARK_PRICE_TIMES_AMM_TO_QUOTE_PRECISION_RATIO, PRICE_SPREAD_PRECISION,
};

pub fn calculate_slippage(
    exit_value: Uint128,
    base_asset_amount: Uint128,
    mark_price_before: i128,
) -> Result<i128, ContractError> {
    let amm_exit_price = exit_value
        .checked_mul(MARK_PRICE_TIMES_AMM_TO_QUOTE_PRECISION_RATIO)?
        .checked_div(base_asset_amount)?;

    Ok((amm_exit_price.u128() as i128)
        .checked_sub(mark_price_before).unwrap_or(0 as i128))
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
