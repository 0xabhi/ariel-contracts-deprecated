use crate::error::ContractError;
use crate::states::market::Market;
use crate::states::order::get_limit_price;

use std::cmp::min;
use std::ops::Div;
use ariel::types::{Order, OrderType, OrderTriggerCondition, PositionDirection};

use crate::helpers::casting::{cast_to_u128};
use crate::helpers::constants::{
    AMM_TO_QUOTE_PRECISION_RATIO, MARK_PRICE_PRECISION,
    MARK_PRICE_TIMES_AMM_TO_QUOTE_PRECISION_RATIO
};
use crate::helpers::amm;

pub fn calculate_base_asset_amount_market_can_execute(
    order: &Order,
    market: &Market,
    precomputed_mark_price: Option<u128>,
    valid_oracle_price: Option<i128>,
) -> Result<u128, ContractError> {
    match order.order_type {
        OrderType::Limit => {
            calculate_base_asset_amount_to_trade_for_limit(order, market, valid_oracle_price)
        }
        OrderType::TriggerMarket => calculate_base_asset_amount_to_trade_for_trigger_market(
            order,
            market,
            precomputed_mark_price,
            valid_oracle_price,
        ),
        OrderType::TriggerLimit => calculate_base_asset_amount_to_trade_for_trigger_limit(
            order,
            market,
            precomputed_mark_price,
            valid_oracle_price,
        ),
        OrderType::Market => Err(ContractError::InvalidOrder),
    }
}

pub fn calculate_base_asset_amount_to_trade_for_limit(
    order: &Order,
    market: &Market,
    valid_oracle_price: Option<i128>,
) -> Result<u128, ContractError> {
    let base_asset_amount_to_fill = order
        .base_asset_amount
        .checked_sub(order.base_asset_amount_filled)
        .ok_or_else(|| (ContractError::MathError))?;

    let limit_price = get_limit_price(order, valid_oracle_price)?;

    let (max_trade_base_asset_amount, max_trade_direction) =
        amm::calculate_max_base_asset_amount_to_trade(&market.amm, limit_price)?;
    if max_trade_direction != order.direction || max_trade_base_asset_amount == 0 {
        return Ok(0);
    }

    let base_asset_amount_to_trade = min(base_asset_amount_to_fill, max_trade_base_asset_amount);

    Ok(base_asset_amount_to_trade)
}

fn calculate_base_asset_amount_to_trade_for_trigger_market(
    order: &Order,
    market: &Market,
    precomputed_mark_price: Option<u128>,
    valid_oracle_price: Option<i128>,
) -> Result<u128, ContractError> {
    let mark_price = match precomputed_mark_price {
        Some(mark_price) => mark_price,
        None => market.amm.mark_price()?,
    };

    match order.trigger_condition {
        OrderTriggerCondition::Above => {
            if mark_price <= order.trigger_price {
                return Ok(0);
            }

            // If there is a valid oracle, check that trigger condition is also satisfied by
            // oracle price (plus some additional buffer)
            if let Some(oracle_price) = valid_oracle_price {
                let oracle_price_101pct = oracle_price
                    .checked_mul(101)
                    .ok_or_else(|| (ContractError::MathError))?
                    .checked_div(100)
                    .ok_or_else(|| (ContractError::MathError))?;

                if cast_to_u128(oracle_price_101pct)? <= order.trigger_price {
                    return Ok(0);
                }
            }
        }
        OrderTriggerCondition::Below => {
            if mark_price >= order.trigger_price {
                return Ok(0);
            }

            // If there is a valid oracle, check that trigger condition is also satisfied by
            // oracle price (plus some additional buffer)
            if let Some(oracle_price) = valid_oracle_price {
                let oracle_price_99pct = oracle_price
                    .checked_mul(99)
                    .ok_or_else(|| (ContractError::MathError))?
                    .checked_div(100)
                    .ok_or_else(|| (ContractError::MathError))?;

                if cast_to_u128(oracle_price_99pct)? >= order.trigger_price {
                    return Ok(0);
                }
            }
        }
    }

    order
        .base_asset_amount
        .checked_sub(order.base_asset_amount_filled)
        .ok_or_else(|| (ContractError::MathError))
}

fn calculate_base_asset_amount_to_trade_for_trigger_limit(
    order: &Order,
    market: &Market,
    precomputed_mark_price: Option<u128>,
    valid_oracle_price: Option<i128>,
) -> Result<u128, ContractError> {
    // if the order has not been filled yet, need to check that trigger condition is met
    if order.base_asset_amount_filled == 0 {
        let base_asset_amount = calculate_base_asset_amount_to_trade_for_trigger_market(
            order,
            market,
            precomputed_mark_price,
            valid_oracle_price,
        )?;
        if base_asset_amount == 0 {
            return Ok(0);
        }
    }

    calculate_base_asset_amount_to_trade_for_limit(order, market, None)
}

pub fn limit_price_satisfied(
    limit_price: u128,
    quote_asset_amount: u128,
    base_asset_amount: u128,
    direction: PositionDirection,
) -> Result<bool, ContractError> {
    let price = quote_asset_amount
        .checked_mul(MARK_PRICE_PRECISION * AMM_TO_QUOTE_PRECISION_RATIO)
        .ok_or_else(|| (ContractError::MathError))?
        .div(base_asset_amount);

    match direction {
        PositionDirection::Long => {
            if price > limit_price {
                return Ok(false);
            }
        }
        PositionDirection::Short => {
            if price < limit_price {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

pub fn calculate_quote_asset_amount_for_maker_order(
    base_asset_amount: u128,
    limit_price: u128,
) -> Result<u128, ContractError> {
    Ok(base_asset_amount
        .checked_mul(limit_price)
        .ok_or_else(|| (ContractError::MathError))?
        .div(MARK_PRICE_TIMES_AMM_TO_QUOTE_PRECISION_RATIO))
}
