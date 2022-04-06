use crate::error::ContractError;
use crate::states::market::Market;
use crate::states::order::get_limit_price;

use std::cell::RefMut;
use std::cmp::min;
use std::ops::Div;
use ariel::types::{Order, OrderType, OrderTriggerCondition, PositionDirection, SwapDirection};

use crate::helpers::amm::calculate_swap_output;
use crate::helpers::casting::{cast, cast_to_i128, cast_to_u128};
use crate::helpers::constants::{
    AMM_TO_QUOTE_PRECISION_RATIO, MARGIN_PRECISION, MARK_PRICE_PRECISION,
    MARK_PRICE_TIMES_AMM_TO_QUOTE_PRECISION_RATIO
};
use crate::controller::margin::calculate_free_collateral;
use crate::helpers::quote_asset::asset_to_reserve_amount;
use crate::states::user::{User, Position};
use crate::helpers::amm;


pub fn calculate_base_asset_amount_user_can_execute(
    user: &mut User,
    user_positions: &mut RefMut<Position>,
    order: &mut Order,
    markets: &mut RefMut<Market>,
    market_index: u64,
) -> Result<u128, ContractError> {
    let position_index = get_position_index(user_positions, market_index)?;

    let quote_asset_amount = calculate_available_quote_asset_user_can_execute(
        user,
        order,
        position_index,
        user_positions,
        markets,
    )?;

    let market = markets.get_market_mut(market_index);

    let order_swap_direction = match order.direction {
        PositionDirection::Long => SwapDirection::Add,
        PositionDirection::Short => SwapDirection::Remove,
    };

    // Extra check in case user have more collateral than market has reserves
    let quote_asset_reserve_amount = min(
        market
            .amm
            .quote_asset_reserve
            .checked_sub(1)
            .ok_or_else(|| (ContractError::MathError))?,
        asset_to_reserve_amount(quote_asset_amount, market.amm.peg_multiplier)?,
    );

    let initial_base_asset_amount = market.amm.base_asset_reserve;
    let (new_base_asset_amount, _new_quote_asset_amount) = calculate_swap_output(
        quote_asset_reserve_amount,
        market.amm.quote_asset_reserve,
        order_swap_direction,
        market.amm.sqrt_k,
    )?;

    let base_asset_amount = cast_to_i128(initial_base_asset_amount)?
        .checked_sub(cast(new_base_asset_amount)?)
        .ok_or_else(|| (ContractError::MathError))?
        .unsigned_abs();

    Ok(base_asset_amount)
}

pub fn calculate_available_quote_asset_user_can_execute(
    user: &User,
    order: &Order,
    position_index: usize,
    user_positions: &mut Position,
    markets: &Market,
) -> Result<u128, ContractError> {
    let market_position = &user_positions.positions[position_index];
    let market = markets.get_market(market_position.market_index);
    let max_leverage = MARGIN_PRECISION
        .checked_div(
            // add one to initial margin ratio so we don't fill exactly to max leverage
            cast_to_u128(market.margin_ratio_initial)?
                .checked_add(1)
                .ok_or_else(|| (ContractError::MathError))?,
        )
        .ok_or_else(|| (ContractError::MathError))?;

    let risk_increasing_in_same_direction = market_position.base_asset_amount == 0
        || market_position.base_asset_amount > 0 && order.direction == PositionDirection::Long
        || market_position.base_asset_amount < 0 && order.direction == PositionDirection::Short;

    let available_quote_asset_for_order = if risk_increasing_in_same_direction {
        let (free_collateral, _) = calculate_free_collateral(user, user_positions, markets, None)?;

        free_collateral
            .checked_mul(max_leverage)
            .ok_or_else(|| (ContractError::MathError))?
    } else {
        let market_index = market_position.market_index;
        let (free_collateral, closed_position_base_asset_value) =
            calculate_free_collateral(user, user_positions, markets, Some(market_index))?;

        free_collateral
            .checked_mul(max_leverage)
            .ok_or_else(|| (ContractError::MathError))?
            .checked_add(closed_position_base_asset_value)
            .ok_or_else(|| (ContractError::MathError))?
    };

    Ok(available_quote_asset_for_order)
}
