use crate::error::ContractError;

use ariel::types::{SwapDirection, PositionDirection};
use cosmwasm_std::Uint128;

use crate::states::market::Amm;
use crate::states::user::Position;

use crate::helpers::amm;
use crate::helpers::amm::calculate_quote_asset_amount_swapped;
use crate::helpers::pnl::calculate_pnl;

use crate::helpers::constants::{AMM_RESERVE_PRECISION, PRICE_TO_QUOTE_PRECISION_RATIO};

pub fn calculate_base_asset_value_and_pnl(
    market_position: &Position,
    a: &Amm,
) -> Result<(Uint128, i128), ContractError> {
    return _calculate_base_asset_value_and_pnl(
        market_position.base_asset_amount,
        market_position.quote_asset_amount,
        a,
    );
}

pub fn _calculate_base_asset_value_and_pnl(
    base_asset_amount: i128,
    quote_asset_amount: Uint128,
    a: &Amm,
) -> Result<(Uint128, i128), ContractError> {
    if base_asset_amount == 0 {
        return Ok((Uint128::zero(), (0 as i128)));
    }

    let swap_direction = swap_direction_to_close_position(base_asset_amount);

    let (new_quote_asset_reserve, _new_base_asset_reserve) = amm::calculate_swap_output(
        Uint128::from(base_asset_amount.unsigned_abs()),
        a.base_asset_reserve,
        swap_direction,
        a.sqrt_k,
    )?;

    let base_asset_value = calculate_quote_asset_amount_swapped(
        a.quote_asset_reserve,
        new_quote_asset_reserve,
        swap_direction,
        a.peg_multiplier,
    )?;

    let pnl = calculate_pnl(base_asset_value, quote_asset_amount, swap_direction)?;

    return Ok((base_asset_value, pnl));
}

pub fn calculate_base_asset_value_and_pnl_with_oracle_price(
    market_position: &Position,
    oracle_price: i128,
) -> Result<(Uint128, i128), ContractError> {
    if market_position.base_asset_amount == 0 {
        return Ok((Uint128::zero(), 0));
    }

    let swap_direction = swap_direction_to_close_position(market_position.base_asset_amount);

    let oracle_price = if oracle_price > 0 {
        Uint128::from(oracle_price.unsigned_abs())
    } else {
        Uint128::zero()
    };

    let base_asset_value = Uint128::from(market_position
        .base_asset_amount
        .unsigned_abs())
        .checked_mul(oracle_price)?
        .checked_div(AMM_RESERVE_PRECISION * PRICE_TO_QUOTE_PRECISION_RATIO)?;

    let pnl = calculate_pnl(
        base_asset_value,
        market_position.quote_asset_amount,
        swap_direction,
    )?;

    Ok((Uint128::from(base_asset_value), pnl))
}

pub fn direction_to_close_position(base_asset_amount: i128) -> PositionDirection {
    if base_asset_amount > 0 {
        PositionDirection::Short
    } else {
        PositionDirection::Long
    }
}

pub fn swap_direction_to_close_position(base_asset_amount: i128) -> SwapDirection {
    if base_asset_amount >= 0 {
        SwapDirection::Add
    } else {
        SwapDirection::Remove
    }
}
