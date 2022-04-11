use cosmwasm_std::{DepsMut, Addr};

use ariel::types::{SwapDirection, PositionDirection};

use crate::error::ContractError;

use crate::helpers::amm::should_round_trade;
use crate::helpers::order::calculate_quote_asset_amount_for_maker_order;
use crate::helpers::position::calculate_base_asset_value_and_pnl;
use crate::states::market::{Market, Markets};
use crate::states::user::{Position, User, Positions, Users};

use crate::helpers::casting::{cast, cast_to_i128};
use crate::helpers::collateral::calculate_updated_collateral;
use crate::helpers::pnl::calculate_pnl;

use crate::controller::amm;

pub fn increase(
    deps: &mut DepsMut,
    direction: PositionDirection,
    quote_asset_amount: u128,
    market_index : u64,
    user_addr: &Addr,
    position_index: u64,
    now: u64,
    precomputed_mark_price: Option<u128>,
) -> Result<i128, ContractError> {
    let mut market = Markets.load(deps.storage, market_index)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    if quote_asset_amount == 0 {
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
        .checked_add(quote_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    let swap_direction = match direction {
        PositionDirection::Long => SwapDirection::Add,
        PositionDirection::Short => SwapDirection::Remove,
    };

    let base_asset_acquired = amm::swap_quote_asset(
        deps,
        market_index,
        quote_asset_amount,
        swap_direction,
        now,
        precomputed_mark_price,
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
    deps: &mut DepsMut,
    direction: PositionDirection,
    quote_asset_swap_amount: u128,
    user_addr: &Addr,
    market_index : u64,
    position_index: u64,
    now: u64,
    precomputed_mark_price: Option<u128>,
) -> Result<i128, ContractError> {
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market = Markets.load(deps.storage, market_index)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
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
    deps: &mut DepsMut,
    user_addr: &Addr,
    market_index : u64,
    position_index: u64,
    now: u64,
    maker_limit_price: Option<u128>,
    precomputed_mark_price: Option<u128>,
) -> Result<(u128, i128, u128), ContractError> {
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market = Markets.load(deps.storage, market_index)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    // If user has no base asset, return early
    if market_position.base_asset_amount == 0 {
        return Ok((0, 0, 0));
    }

    let swap_direction = if market_position.base_asset_amount > 0 {
        SwapDirection::Add
    } else {
        SwapDirection::Remove
    };

    let quote_asset_swapped = amm::swap_base_asset(
        deps, 
        market_index,
        market_position.base_asset_amount.unsigned_abs(),
        swap_direction,
        now,
        precomputed_mark_price
    )?;

    let (quote_asset_amount, quote_asset_amount_surplus) = match maker_limit_price {	
        Some(limit_price) => calculate_quote_asset_amount_surplus(	
            swap_direction,	
            quote_asset_swapped,	
            market_position.base_asset_amount.unsigned_abs(),	
            limit_price,	
        )?,	
        None => (quote_asset_swapped, 0),	
    };

    let pnl = calculate_pnl(
        quote_asset_swapped,
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

    Ok((
        quote_asset_amount,
        base_asset_amount,
        quote_asset_amount_surplus,
    ))
}

pub fn add_new_position(
    deps: &mut DepsMut,
    user_addr: &Addr,
    market_index: u64,
) -> Result<u64, ContractError> {
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market = Markets.load(deps.storage, market_index)?;
    
    let new_position_index = user.positions_length
        .checked_add(1)
        .ok_or_else(|| (ContractError::MaxNumberOfPositions))?;

    let new_market_position = Position {
        market_index,
        base_asset_amount: 0,
        quote_asset_amount: 0,
        last_cumulative_funding_rate: 0,
        last_cumulative_repeg_rebate: 0,
        last_funding_rate_ts: 0,
        order_length: 0,
    };

    Positions.update(deps.storage, (user_addr, new_position_index), |p|-> Result<Position, ContractError> {
        Ok(new_market_position)
    })?;

    user.positions_length = new_position_index;

    Users.update(deps.storage, user_addr, |u|-> Result<User, ContractError> {
        Ok(user)
    })?;
    
    Ok(new_position_index)
}

pub fn increase_with_base_asset_amount(
    deps: &mut DepsMut,
    direction: PositionDirection,
    base_asset_amount: u128,
    user_addr: &Addr,
    position_index: u64,
    now: u64,
    maker_limit_price: Option<u128>,
    precomputed_mark_price: Option<u128>,
) -> Result<(u128, u128), ContractError> {

    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let market_index = market_position.market_index;
    let mut market = Markets.load(deps.storage, market_index)?;
    
    if base_asset_amount == 0 {
        return Ok((0, 0));
    }
    
    let mut market = Markets.load(deps.storage, market_index)?;
    
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

    let swap_direction = match direction {
        PositionDirection::Long => SwapDirection::Remove,
        PositionDirection::Short => SwapDirection::Add,
    };

    let quote_asset_swapped = amm::swap_base_asset(
        deps,
        market_index,
        base_asset_amount,
        swap_direction,
        now,
        precomputed_mark_price,
    )?;

    let (quote_asset_amount, quote_asset_amount_surplus) = match maker_limit_price {
        Some(limit_price) => calculate_quote_asset_amount_surplus(
            swap_direction,
            quote_asset_swapped,
            base_asset_amount,
            limit_price,
        )?,
        None => (quote_asset_swapped, 0),
    };

    market_position.quote_asset_amount = market_position
        .quote_asset_amount
        .checked_add(quote_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    let base_asset_amount = match direction {
        PositionDirection::Long => cast_to_i128(base_asset_amount)?,
        PositionDirection::Short => -cast_to_i128(base_asset_amount)?,
    };

    market_position.base_asset_amount = market_position
        .base_asset_amount
        .checked_add(base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;
    market.base_asset_amount = market
        .base_asset_amount
        .checked_add(base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    if market_position.base_asset_amount > 0 {
        market.base_asset_amount_long = market
            .base_asset_amount_long
            .checked_add(base_asset_amount)
            .ok_or_else(|| (ContractError::MathError))?;
    } else {
        market.base_asset_amount_short = market
            .base_asset_amount_short
            .checked_add(base_asset_amount)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    Markets.update(deps.storage, market_index, |m| ->  Result<Market, ContractError>{
        Ok(market)
    });

    Positions.update(deps.storage, (user_addr, position_index), |p| -> Result<Position, ContractError> {
        Ok(market_position)
    })?;

    Users.update(deps.storage, user_addr, |u|-> Result<User, ContractError> {
        Ok(user)
    })?;

    Ok((quote_asset_amount, quote_asset_amount_surplus))
}

pub fn reduce_with_base_asset_amount(
    deps: &mut DepsMut,
    direction: PositionDirection,
    base_asset_amount: u128,
    user_addr: &Addr,
    position_index: u64,
    now: u64,
    maker_limit_price: Option<u128>,
    precomputed_mark_price: Option<u128>,
) -> Result<(u128, u128), ContractError> {

    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let market_index = market_position.market_index;
    let mut market = Markets.load(deps.storage, market_index)?;
    

    let swap_direction = match direction {
        PositionDirection::Long => SwapDirection::Remove,
        PositionDirection::Short => SwapDirection::Add,
    };

    let quote_asset_swapped = amm::swap_base_asset(
        deps,
        market_index,
        base_asset_amount,
        swap_direction,
        now,
        precomputed_mark_price,
    )?;

    let (quote_asset_amount, quote_asset_amount_surplus) = match maker_limit_price {
        Some(limit_price) => calculate_quote_asset_amount_surplus(
            swap_direction,
            quote_asset_swapped,
            base_asset_amount,
            limit_price,
        )?,
        None => (quote_asset_swapped, 0),
    };

    let base_asset_amount = match direction {
        PositionDirection::Long => cast_to_i128(base_asset_amount)?,
        PositionDirection::Short => -cast_to_i128(base_asset_amount)?,
    };

    let base_asset_amount_before = market_position.base_asset_amount;
    market_position.base_asset_amount = market_position
        .base_asset_amount
        .checked_add(base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    market.open_interest = market
        .open_interest
        .checked_sub(cast(market_position.base_asset_amount == 0)?)
        .ok_or_else(|| (ContractError::MathError))?;
    market.base_asset_amount = market
        .base_asset_amount
        .checked_add(base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    if market_position.base_asset_amount > 0 {
        market.base_asset_amount_long = market
            .base_asset_amount_long
            .checked_add(base_asset_amount)
            .ok_or_else(|| (ContractError::MathError))?;
    } else {
        market.base_asset_amount_short = market
            .base_asset_amount_short
            .checked_add(base_asset_amount)
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

    let pnl = if PositionDirection::Short == direction {
        cast_to_i128(quote_asset_amount)?
            .checked_sub(cast(initial_quote_asset_amount_closed)?)
            .ok_or_else(|| (ContractError::MathError))?
    } else {
        cast_to_i128(initial_quote_asset_amount_closed)?
            .checked_sub(cast(quote_asset_amount)?)
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


    Ok((quote_asset_amount, quote_asset_amount_surplus))
}

pub fn update_position_with_base_asset_amount(
    deps: &mut DepsMut,
    base_asset_amount: u128,
    direction: PositionDirection,
    user_addr: &Addr,
    position_index: u64,
    mark_price_before: u128,
    now: u64,
    maker_limit_price: Option<u128>,
) -> Result<(bool, bool, u128, u128, u128), ContractError> {
    
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let market_index = market_position.market_index;
    let mut market = Markets.load(deps.storage, market_index)?;
    
    // A trade is risk increasing if it increases the users leverage
    // If a trade is risk increasing and brings the user's margin ratio below initial requirement
    // the trade fails
    // If a trade is risk increasing and it pushes the mark price too far away from the oracle price
    // the trade fails
    let mut potentially_risk_increasing = true;
    let mut reduce_only = false;

    // The trade increases the the user position if
    // 1) the user does not have a position
    // 2) the trade is in the same direction as the user's existing position
    let quote_asset_amount;
    let quote_asset_amount_surplus;
    let increase_position = market_position.base_asset_amount == 0
        || market_position.base_asset_amount > 0 && direction == PositionDirection::Long
        || market_position.base_asset_amount < 0 && direction == PositionDirection::Short;
    if increase_position {
        let (_quote_asset_amount, _quote_asset_amount_surplus) = increase_with_base_asset_amount(
            deps,
            direction,
            base_asset_amount,
            user_addr,
            position_index,
            now,
            maker_limit_price,
            Some(mark_price_before),
        )?;
        quote_asset_amount = _quote_asset_amount;
        quote_asset_amount_surplus = _quote_asset_amount_surplus;
    } else if market_position.base_asset_amount.unsigned_abs() > base_asset_amount {
        let (_quote_asset_amount, _quote_asset_amount_surplus) = reduce_with_base_asset_amount(
            deps,
            direction,
            base_asset_amount,
            user_addr,
            position_index,
            now,
            maker_limit_price,
            Some(mark_price_before),
        )?;
        quote_asset_amount = _quote_asset_amount;
        quote_asset_amount_surplus = _quote_asset_amount_surplus;

        reduce_only = true;
        potentially_risk_increasing = false;
    } else {
        // after closing existing position, how large should trade be in opposite direction
        let base_asset_amount_after_close = base_asset_amount
            .checked_sub(market_position.base_asset_amount.unsigned_abs())
            .ok_or_else(|| (ContractError::MathError))?;

        // If the value of the new position is less than value of the old position, consider it risk decreasing
        if base_asset_amount_after_close < market_position.base_asset_amount.unsigned_abs() {
            potentially_risk_increasing = false;
        }

        let (quote_asset_amount_closed, _,  quote_asset_amount_surplus_closed) =
            close(
                deps,
                user_addr, 
                market_index, 
                position_index, 
                now, 
                maker_limit_price,
                Some(mark_price_before),     
            )?;

        let (quote_asset_amount_opened, quote_asset_amount_surplus_opened) =
            increase_with_base_asset_amount(
                deps,
                direction,
                base_asset_amount_after_close,
                user_addr,
                position_index,
                now,
                maker_limit_price,
                Some(mark_price_before),
            )?;

        // means position was closed and it was reduce only
        if quote_asset_amount_opened == 0 {
            reduce_only = true;
        }

        quote_asset_amount = quote_asset_amount_closed
            .checked_add(quote_asset_amount_opened)
            .ok_or_else(|| (ContractError::MathError))?;

        quote_asset_amount_surplus = quote_asset_amount_surplus_closed
            .checked_add(quote_asset_amount_surplus_opened)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    Ok((
        potentially_risk_increasing,
        reduce_only,
        base_asset_amount,
        quote_asset_amount,
        quote_asset_amount_surplus,
    ))
}

pub fn update_position_with_quote_asset_amount(
    deps: &mut DepsMut,
    quote_asset_amount: u128,
    direction: PositionDirection,
    user_addr: &Addr,
    position_index: u64,
    mark_price_before: u128,
    now: u64,
) -> Result<(bool, bool, u128, u128, u128), ContractError> {
 
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let market_index = market_position.market_index;
    let mut market = Markets.load(deps.storage, market_index)?;
  
    
    // A trade is risk increasing if it increases the users leverage
    // If a trade is risk increasing and brings the user's margin ratio below initial requirement
    // the trade fails
    // If a trade is risk increasing and it pushes the mark price too far away from the oracle price
    // the trade fails
    let mut potentially_risk_increasing = true;
    let mut reduce_only = false;

    let mut quote_asset_amount = quote_asset_amount;
    let base_asset_amount;
    // The trade increases the the user position if
    // 1) the user does not have a position
    // 2) the trade is in the same direction as the user's existing position
    let increase_position = market_position.base_asset_amount == 0
        || market_position.base_asset_amount > 0 && direction == PositionDirection::Long
        || market_position.base_asset_amount < 0 && direction == PositionDirection::Short;
    if increase_position {
        base_asset_amount = increase(
            deps,
            direction,
            quote_asset_amount,
            market_index,
            user_addr,
            position_index,
            now,
            Some(mark_price_before),
        )?
        .unsigned_abs();
    } else {
        let (base_asset_value, _unrealized_pnl) =
            calculate_base_asset_value_and_pnl(&market_position, &market.amm)?;

        // if the quote_asset_amount is close enough in value to base_asset_value,
        // round the quote_asset_amount to be the same as base_asset_value
        if should_round_trade(&market.amm, quote_asset_amount, base_asset_value)? {
            quote_asset_amount = base_asset_value;
        }

        // we calculate what the user's position is worth if they closed to determine
        // if they are reducing or closing and reversing their position
        if base_asset_value > quote_asset_amount {
            base_asset_amount = reduce(
                deps,
                direction,
                quote_asset_amount,
                user_addr,
                market_index,
                position_index,
                now,
                Some(mark_price_before),
            )?
            .unsigned_abs();

            potentially_risk_increasing = false;
            reduce_only = true;
        } else {
            // after closing existing position, how large should trade be in opposite direction
            let quote_asset_amount_after_close = quote_asset_amount
                .checked_sub(base_asset_value)
                .ok_or_else(|| (ContractError::MathError))?;

            // If the value of the new position is less than value of the old position, consider it risk decreasing
            if quote_asset_amount_after_close < base_asset_value {
                potentially_risk_increasing = false;
            }

            let (_, base_asset_amount_closed, _) = close(
                deps,
                user_addr,
                market_index,
                position_index,
                now,
                None,
                Some(mark_price_before),
            )?;
            let base_asset_amount_closed = base_asset_amount_closed.unsigned_abs();

            let base_asset_amount_opened = increase(
                deps,
                direction,
                quote_asset_amount_after_close,
                market_index,
                user_addr,
                position_index,
                now,
                Some(mark_price_before),
            )?
            .unsigned_abs();

            // means position was closed and it was reduce only
            if base_asset_amount_opened == 0 {
                reduce_only = true;
            }

            base_asset_amount = base_asset_amount_closed
                .checked_add(base_asset_amount_opened)
                .ok_or_else(|| (ContractError::MathError))?;
        }
    }

    Ok((
        potentially_risk_increasing,
        reduce_only,
        base_asset_amount,
        quote_asset_amount,
        0,
    ))
}

fn calculate_quote_asset_amount_surplus(
    swap_direction: SwapDirection,
    quote_asset_swapped: u128,
    base_asset_amount: u128,
    limit_price: u128,
) -> Result<(u128, u128), ContractError> {
    let quote_asset_amount =
        calculate_quote_asset_amount_for_maker_order(base_asset_amount, limit_price)?;

    let quote_asset_amount_surplus = match swap_direction {
        SwapDirection::Remove => quote_asset_amount
            .checked_sub(quote_asset_swapped)
            .ok_or_else(|| (ContractError::MathError))?,
        SwapDirection::Add => quote_asset_swapped
            .checked_sub(quote_asset_amount)
            .ok_or_else(|| (ContractError::MathError))?,
    };

    Ok((quote_asset_amount, quote_asset_amount_surplus))
}
