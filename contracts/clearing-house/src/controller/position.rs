use cosmwasm_std::{DepsMut, Addr};

use ariel::types::{SwapDirection, PositionDirection};

use crate::error::ContractError;

use crate::states::market::{Market, Markets};
use crate::states::user::{Position, User, Positions, Users};

use crate::helpers::casting::{cast, cast_to_i128};
use crate::helpers::collateral::calculate_updated_collateral;
use crate::helpers::pnl::calculate_pnl;

use crate::controller::amm;

pub fn increase(
    deps: DepsMut,
    direction: PositionDirection,
    new_quote_asset_notional_amount: u128,
    market_index : u64,
    user_addr: &Addr,
    position_index: u64,
    now: i64,
) -> Result<i128, ContractError> {
    let market = Markets.load(deps.storage, market_index)?;
    let market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    if new_quote_asset_notional_amount == 0 {
        return Ok(0);
    }

    // Update funding rate if this is a new position
    if market_position.base_asset_amount == 0 {
        market_position.last_cumulative_funding_rate = match direction {
            PositionDirection::Long => market.amm.cumulative_funding_rate_long,
            PositionDirection::Short => market.amm.cumulative_funding_rate_short,
        };

        market.open_interest = market
            .open_interest
            .checked_add(1)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    market_position.quote_asset_amount = market_position
        .quote_asset_amount
        .checked_add(new_quote_asset_notional_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    let swap_direction = match direction {
        PositionDirection::Long => SwapDirection::Add,
        PositionDirection::Short => SwapDirection::Remove,
    };

    let base_asset_acquired = amm::swap_quote_asset(
        deps,
        market_index,
        new_quote_asset_notional_amount,
        swap_direction,
        now,
        None,
    )?;

    // update the position size on market and user
    market_position.base_asset_amount = market_position
        .base_asset_amount
        .checked_add(base_asset_acquired)
        .ok_or_else(|| (ContractError::MathError))?;
    market.base_asset_amount = market
        .base_asset_amount
        .checked_add(base_asset_acquired)
        .ok_or_else(|| (ContractError::MathError))?;

    if market_position.base_asset_amount > 0 {
        market.base_asset_amount_long = market
            .base_asset_amount_long
            .checked_add(base_asset_acquired)
            .ok_or_else(|| (ContractError::MathError))?;
    } else {
        market.base_asset_amount_short = market
            .base_asset_amount_short
            .checked_add(base_asset_acquired)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    Markets.update(deps.storage, market_index, |m| ->  Result<Market, ContractError>{
        Ok(market)
    });

    Positions.update(deps.storage, (user_addr, market_index), |p| -> Result<Position, ContractError> {
        Ok(market_position)
    })?;

    Ok(base_asset_acquired)
}

pub fn reduce(
    deps: DepsMut,
    direction: PositionDirection,
    quote_asset_swap_amount: u128,
    user_addr: &Addr,
    market_index : u64,
    position_index: u64,
    now: i64,
    precomputed_mark_price: Option<u128>,
) -> Result<i128, ContractError> {
    let user = Users.load(deps.storage, user_addr)?;
    let market = Markets.load(deps.storage, market_index)?;
    let market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let swap_direction = match direction {
        PositionDirection::Long => SwapDirection::Add,
        PositionDirection::Short => SwapDirection::Remove,
    };

    let base_asset_swapped = amm::swap_quote_asset(
        deps, 
        market_index,
        quote_asset_swap_amount,
        swap_direction,
        now,
        precomputed_mark_price,
    )?;

    let base_asset_amount_before = market_position.base_asset_amount;
    market_position.base_asset_amount = market_position
        .base_asset_amount
        .checked_add(base_asset_swapped)
        .ok_or_else(|| (ContractError::MathError))?;

    market.open_interest = market
        .open_interest
        .checked_sub(cast(market_position.base_asset_amount == 0)?)
        .ok_or_else(|| (ContractError::MathError))?;
    market.base_asset_amount = market
        .base_asset_amount
        .checked_add(base_asset_swapped)
        .ok_or_else(|| (ContractError::MathError))?;

    if market_position.base_asset_amount > 0 {
        market.base_asset_amount_long = market
            .base_asset_amount_long
            .checked_add(base_asset_swapped)
            .ok_or_else(|| (ContractError::MathError))?;
    } else {
        market.base_asset_amount_short = market
            .base_asset_amount_short
            .checked_add(base_asset_swapped)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    let base_asset_amount_change = base_asset_amount_before
        .checked_sub(market_position.base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?
        .abs();

    let initial_quote_asset_amount_closed = market_position
        .quote_asset_amount
        .checked_mul(base_asset_amount_change.unsigned_abs())
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(base_asset_amount_before.unsigned_abs())
        .ok_or_else(|| (ContractError::MathError))?;

    market_position.quote_asset_amount = market_position
        .quote_asset_amount
        .checked_sub(initial_quote_asset_amount_closed)
        .ok_or_else(|| (ContractError::MathError))?;

    let pnl = if market_position.base_asset_amount > 0 {
        cast_to_i128(quote_asset_swap_amount)?
            .checked_sub(cast(initial_quote_asset_amount_closed)?)
            .ok_or_else(|| (ContractError::MathError))?
    } else {
        cast_to_i128(initial_quote_asset_amount_closed)?
            .checked_sub(cast(quote_asset_swap_amount)?)
            .ok_or_else(|| (ContractError::MathError))?
    };

    user.collateral = calculate_updated_collateral(user.collateral, pnl)?;

    Markets.update(deps.storage, market_index, |m| ->  Result<Market, ContractError>{
        Ok(market)
    });

    Positions.update(deps.storage, (user_addr, position_index), |p| -> Result<Position, ContractError> {
        Ok(market_position)
    })?;

    Users.update(deps.storage, user_addr, |u|-> Result<User, ContractError> {
        Ok(user)
    })?;

    Ok(base_asset_swapped)
}

pub fn close(
    deps: DepsMut,
    user_addr: &Addr,
    market_index : u64,
    position_index: u64,
    now: i64,
) -> Result<(u128, i128), ContractError> {
    let user = Users.load(deps.storage, user_addr)?;
    let market = Markets.load(deps.storage, market_index)?;
    let market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    // If user has no base asset, return early
    if market_position.base_asset_amount == 0 {
        return Ok((0, 0));
    }

    let swap_direction = if market_position.base_asset_amount > 0 {
        SwapDirection::Add
    } else {
        SwapDirection::Remove
    };

    let base_asset_value = amm::swap_base_asset(
        deps, 
        market_index,
        market_position.base_asset_amount.unsigned_abs(),
        swap_direction,
        now,
    )?;
    let pnl = calculate_pnl(
        base_asset_value,
        market_position.quote_asset_amount,
        swap_direction,
    )?;

    user.collateral = calculate_updated_collateral(user.collateral, pnl)?;
    market_position.last_cumulative_funding_rate = 0;
    market_position.last_funding_rate_ts = 0;

    market.open_interest = market
        .open_interest
        .checked_sub(1)
        .ok_or_else(|| (ContractError::MathError))?;

    market_position.quote_asset_amount = 0;

    market.base_asset_amount = market
        .base_asset_amount
        .checked_sub(market_position.base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    if market_position.base_asset_amount > 0 {
        market.base_asset_amount_long = market
            .base_asset_amount_long
            .checked_sub(market_position.base_asset_amount)
            .ok_or_else(|| (ContractError::MathError))?;
    } else {
        market.base_asset_amount_short = market
            .base_asset_amount_short
            .checked_sub(market_position.base_asset_amount)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    let base_asset_amount = market_position.base_asset_amount;
    market_position.base_asset_amount = 0;

    Markets.update(deps.storage, market_index, |m| ->  Result<Market, ContractError>{
        Ok(market)
    });

    Positions.update(deps.storage, (user_addr, position_index), |p| -> Result<Position, ContractError> {
        Ok(market_position)
    })?;

    Users.update(deps.storage, user_addr, |u|-> Result<User, ContractError> {
        Ok(user)
    })?;

    Ok((base_asset_value, base_asset_amount))
}
