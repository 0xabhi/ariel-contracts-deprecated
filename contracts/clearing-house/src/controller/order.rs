use crate::error::ContractError;
use crate::helpers::collateral::calculate_updated_collateral;
use crate::helpers::fees::calculate_order_fee_tier;
use crate::helpers::order::{validate_order, validate_order_can_be_canceled, calculate_base_asset_amount_market_can_execute};
use crate::states::market::{Markets};
use crate::states::order::{Orders, OrderParams, OrderState};
use crate::states::order_history::{OrderRecord, OrderAction, OrderHistoryInfo, OrderHistory, OrderHisInfo};
use crate::states::state::{STATE};

use crate::helpers::order::get_valid_oracle_price;
use std::cmp::min;
use ariel::types::{Order, OrderType, PositionDirection, SwapDirection, OrderStatus};
use cosmwasm_std::{DepsMut, Addr};

use crate::helpers::amm::{calculate_swap_output};
use crate::helpers::casting::{cast, cast_to_i128, cast_to_u128};
use crate::helpers::constants::{
    MARGIN_PRECISION, QUOTE_PRECISION
};
use crate::controller::margin::calculate_free_collateral;
use crate::helpers::quote_asset::asset_to_reserve_amount;
use crate::states::user::{Users, Positions, Position};
use crate::helpers::amm;

use crate::controller::funding::settle_funding_payment;

use super::position::{update_position_with_base_asset_amount};

pub fn calculate_base_asset_amount_user_can_execute(
    deps: &mut DepsMut,
    user_addr: &Addr,
    order_index: u64,
    market_index: u64,
) -> Result<u128, ContractError> {

    let mut user = Users.load(deps.storage, user_addr)?;
    let position_index = market_index;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    let mut market = Markets.load(deps.storage, market_index)?;
    
    let order = Orders.load(deps.storage, ((user_addr, market_index), order_index))?;

    let quote_asset_amount = calculate_available_quote_asset_user_can_execute(
        deps,
        user_addr,
        order_index,
        position_index,
    )?;

    
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
    deps: &DepsMut,
    user_addr: &Addr,
    order_index: u64,
    position_index: u64,
) -> Result<u128, ContractError> {

    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let market_index = market_position.market_index;
    let mut market = Markets.load(deps.storage, market_index)?;
    
    let order = Orders.load(deps.storage, ((user_addr, position_index), order_index))?;

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
        let (free_collateral, _) = calculate_free_collateral(
            deps,
            user_addr,
            None, 
        )?;

        free_collateral
            .checked_mul(max_leverage)
            .ok_or_else(|| (ContractError::MathError))?
    } else {
        let market_index = market_position.market_index;
        let (free_collateral, closed_position_base_asset_value) =
            calculate_free_collateral(deps, user_addr, Some(market_index))?;

        free_collateral
            .checked_mul(max_leverage)
            .ok_or_else(|| (ContractError::MathError))?
            .checked_add(closed_position_base_asset_value)
            .ok_or_else(|| (ContractError::MathError))?
    };

    Ok(available_quote_asset_for_order)
}

pub fn place_order(
    deps: &mut DepsMut,
    user_addr: &Addr,
    order_index: u64,
    now: u64,
    params: OrderParams,
    oracle: &Addr,
    order_state: &OrderState,
) -> Result<bool, ContractError> {

    let state = STATE.load(deps.storage)?;

    let mut user = Users.load(deps.storage, user_addr)?;
    let position_index = params.market_index;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let market_index = market_position.market_index;
    let mut market = Markets.load(deps.storage, market_index)?;
   
    settle_funding_payment(
        deps,
        user_addr,
        now,
    )?;
    
    let discount_tier = calculate_order_fee_tier(
        &state.fee_structure,
        params.base_asset_amount,
    )?;

    // Increment open orders for existing position
    market_position.order_length = market_position.order_length.checked_add(1).ok_or_else(|| (ContractError::MathError))?;

    let new_order_idx = market_position.order_length;

    let new_order = Order {
        status: OrderStatus::Open,
        order_type: params.order_type,
        ts: now,
        position_index,
        market_index,
        price: params.price,
        user_base_asset_amount: market_position.base_asset_amount,
        base_asset_amount: params.base_asset_amount,
        quote_asset_amount: params.quote_asset_amount,
        base_asset_amount_filled: 0,
        quote_asset_amount_filled: 0,
        fee: 0,
        direction: params.direction,
        reduce_only: params.reduce_only,
        discount_tier,
        trigger_price: params.trigger_price,
        trigger_condition: params.trigger_condition,
        referrer: match user.referrer {
            Some(referrer) => referrer,
            None => Addr::unchecked(""),
        },
        post_only: params.post_only,
        oracle_price_offset: params.oracle_price_offset,
        // always false until we add support
        immediate_or_cancel: false,
    };

    Orders.save(deps.storage, ((user_addr, position_index), new_order_idx),&new_order);

    let valid_oracle_price = get_valid_oracle_price(
        *oracle,
        &market,
        &new_order,
        &state.oracle_guard_rails,
    )?;

    validate_order(
        &new_order, 
        &market, 
        order_state, 
        valid_oracle_price
    )?;

    // Add to the order history account    user.order_length = new_order_idx;
    let order_history_info_length = 
    OrderHistoryInfo.load(deps.storage)?
    .len.checked_add(1).ok_or_else(|| (ContractError::MathError))?;
    OrderHistoryInfo.update(deps.storage, |mut i|-> Result<OrderHisInfo, ContractError> {
        i.len = order_history_info_length;
        Ok(i)
    })?;
    OrderHistory.save(deps.storage, order_history_info_length, &OrderRecord {
        ts: now,
        order: new_order,
        user: *user_addr,
        action: OrderAction::Place,
        filler: Addr::unchecked(""),
        trade_record_id: 0,
        base_asset_amount_filled: 0,
        quote_asset_amount_filled: 0,
        filler_reward: 0,
        fee: 0,
        quote_asset_amount_surplus: 0,
        position_index,
    })?;

    Ok(true)
}

pub fn cancel_order(
    deps: &mut DepsMut,
    user_addr: &Addr,
    position_index: u64,
    order_index: u64,
    oracle: Addr,
    now: u64
) -> Result<bool, ContractError> {

    let state = STATE.load(deps.storage)?;
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut market_position = Positions.load(deps.storage, (user_addr, position_index))?;
    
    let mut order = Orders.load(deps.storage, ((user_addr, position_index), order_index))?;
    let market = Markets.load(deps.storage, position_index)?;

    settle_funding_payment(
        deps,
        user_addr, 
        now
    )?;

    if order.status != OrderStatus::Open {
        return Err(ContractError::OrderNotOpen);
    }

    let valid_oracle_price = get_valid_oracle_price(
        oracle,
        &market,
        &order,
        &state.oracle_guard_rails,
    )?;

    validate_order_can_be_canceled(
        &order,
        &market,
        valid_oracle_price,
    )?;

    // Add to the order history account
    let order_history_info_length = 
    OrderHistoryInfo.load(deps.storage)?
    .len.checked_add(1).ok_or_else(|| (ContractError::MathError))?;
    OrderHistoryInfo.update(deps.storage, |mut i|-> Result<OrderHisInfo, ContractError> {
        i.len = order_history_info_length;
        Ok(i)
    })?;
    OrderHistory.save(deps.storage, order_history_info_length, &OrderRecord {
        ts: now,
        user: *user_addr,
        order: order,
        action: OrderAction::Cancel,
        filler: Addr::unchecked(""),
        trade_record_id: 0,
        base_asset_amount_filled: 0,
        quote_asset_amount_filled: 0,
        fee: 0,
        filler_reward: 0,
        quote_asset_amount_surplus: 0,
        position_index,
    })?;

    if order_index != market_position.order_length {
        let order_to_replace = Orders.load(deps.storage, ((user_addr, position_index), market_position.order_length))?;
        Orders.update(deps.storage, ((user_addr, position_index), order_index), |p| -> Result<Order, ContractError> {
            Ok(order_to_replace)
        })?;
    }
    
    Orders.remove(deps.storage, ((user_addr, position_index), market_position.order_length));

    // Decrement open orders for existing position
    market_position.order_length -= 1;
    Positions.update(deps.storage, (user_addr, position_index), |p| -> Result<Position, ContractError> {
        Ok(market_position)
    })?;

    Ok(true)
}


pub fn expire_orders(
    deps: &DepsMut,
    user_addr: &Addr,
    now: u64,
    filler_addr: &Addr,
) -> Result<bool, ContractError> {
    let state = STATE.load(deps.storage)?;
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut filler = Users.load(deps.storage, filler_addr)?;
    
    let ten_quote = 10 * QUOTE_PRECISION;

    if user.collateral >= ten_quote {
        // msg!("User has more than ten quote asset, cant expire orders");
        return Err(ContractError::CantExpireOrders);
    }

    let max_filler_reward = QUOTE_PRECISION / 100; // .01 quote asset
    let filler_reward = min(user.collateral, max_filler_reward);

    user.collateral = calculate_updated_collateral(user.collateral, -(filler_reward as i128))?;
    filler.collateral = calculate_updated_collateral(filler.collateral, filler_reward as i128)?;

    let expired_order_len: u64 = 0;
    if state.markets_length > 0 {
        for i in 1..state.markets_length {
            let market = Markets.load(deps.storage, i);
            let market_position = Positions.load(deps.storage, (user_addr,i));
            let something = match market_position {
                Ok(p) => {
                    if p.order_length > 0 {
                        for j in 1..p.order_length {
                            let order = Orders.load(deps.storage, ((user_addr, i), j))?;
                            if order.status == OrderStatus::Open {
                                expired_order_len += 1;
                            }
                        }
                    };
                    Ok(true)
                },
                Err(_) => Ok(true),
            };
        }
    }
    let filler_reward_per_order: u128 = filler_reward / (expired_order_len as u128);

    if state.markets_length > 0 {
        for i in 1..state.markets_length {
            let market = Markets.load(deps.storage, i);
            let market_position = Positions.load(deps.storage, (user_addr,i))?;
            let something = match market_position {
                Ok(p) => {
                    if p.order_length > 0 {
                        for j in 1..p.order_length {
                            let order = Orders.load(deps.storage, ((user_addr, i), j))?;
                            if order.status == OrderStatus::Init {
                                continue;
                            }
                            order.fee = order
                            .fee
                            .checked_add(filler_reward_per_order)
                            .ok_or_else(|| (ContractError::MathError))?;

                            // Add to the order history account
                            let order_history_info_length = 
                            OrderHistoryInfo.load(deps.storage)?
                            .len.checked_add(1).ok_or_else(|| (ContractError::MathError))?;
                            OrderHistoryInfo.update(deps.storage, |mut k|-> Result<OrderHisInfo, ContractError> {
                                k.len = order_history_info_length;
                                Ok(k)
                            })?;
                            OrderHistory.save(deps.storage, order_history_info_length, &OrderRecord {
                                ts: now,
                                order: order,
                                user: *user_addr,
                                action: OrderAction::Expire,
                                filler: *filler_addr,
                                trade_record_id: 0,
                                base_asset_amount_filled: 0,
                                quote_asset_amount_filled: 0,
                                filler_reward: filler_reward_per_order,
                                fee: filler_reward_per_order,
                                quote_asset_amount_surplus: 0,
                                position_index : i,
                            })?;

                            if j != market_position.order_length {
                                let order_to_replace = Orders.load(deps.storage, ((user_addr, i), market_position.order_length))?;
                                Orders.update(deps.storage, ((user_addr, i), j), |p| -> Result<Order, ContractError> {
                                    Ok(order_to_replace)
                                })?;
                            }
                            
                            Orders.remove(deps.storage, ((user_addr, i), market_position.order_length));

                            market_position.order_length -= 1;
                            // Decrement open orders for existing position
                            Positions.update(deps.storage, (user_addr, i), |p| -> Result<Position, ContractError> {
                                Ok(market_position)
                            })?;
                            
                            j -= 1;
                        }
                    };
                    Ok(true)
                },
                Err(_) => Ok(true),
            };
        }
    }

    Ok(true)
}
 
pub fn fill_order(
    deps: &DepsMut,
    user_addr: &Addr,
    filler_addr: &Addr,
    position_index: u64,
    order_index: u64,
    now: u64,
) -> Result<u128, ContractError> {
    let state = STATE.load(deps.storage)?;
    let mut user = Users.load(deps.storage, user_addr)?;
    let mut filler = Users.load(deps.storage, filler_addr)?;
    
    {
        settle_funding_payment(
            deps,
            user_addr,
            now,
        )?;
    }

    let user_orders = &mut user_orders
        .load_mut()
        .or(Err(ContractError::UnableToLoadAccountLoader))?;
    let order_index = user_orders
        .orders
        .iter()
        .position(|order| order.order_id == order_id)
        .ok_or_else(print_error!(ContractError::OrderDoesNotExist))?;
    let order = &mut user_orders.orders[order_index];

    if order.status != OrderStatus::Open {
        return Err(ContractError::OrderNotOpen);
    }

    let market_index = order.market_index;
    {
        let markets = &markets
            .load()
            .or(Err(ContractError::UnableToLoadAccountLoader))?;
        let market = markets.get_market(market_index);

        if !market.initialized {
            return Err(ContractError::MarketIndexNotInitialized);
        }

        if !market.amm.oracle.eq(oracle.key) {
            return Err(ContractError::InvalidOracle);
        }
    }

    let mark_price_before: u128;
    let oracle_mark_spread_pct_before: i128;
    let is_oracle_valid: bool;
    let oracle_price: i128;
    {
        let markets = &mut markets
            .load_mut()
            .or(Err(ContractError::UnableToLoadAccountLoader))?;
        let market = markets.get_market_mut(market_index);
        mark_price_before = market.amm.mark_price()?;
        let oracle_price_data = &market.amm.get_oracle_price(oracle, clock_slot)?;
        oracle_mark_spread_pct_before = amm::calculate_oracle_mark_spread_pct(
            &market.amm,
            oracle_price_data,
            Some(mark_price_before),
        )?;
        oracle_price = oracle_price_data.price;
        let normalised_price =
            normalise_oracle_price(&market.amm, oracle_price_data, Some(mark_price_before))?;
        is_oracle_valid = amm::is_oracle_valid(
            &market.amm,
            oracle_price_data,
            &state.oracle_guard_rails.validity,
        )?;
        if is_oracle_valid {
            amm::update_oracle_price_twap(&mut market.amm, now, normalised_price)?;
        }
    }

    let valid_oracle_price = if is_oracle_valid {
        Some(oracle_price)
    } else {
        None
    };

    let (
        base_asset_amount,
        quote_asset_amount,
        potentially_risk_increasing,
        quote_asset_amount_surplus,
    ) = execute_order(
        user,
        user_positions,
        order,
        &mut markets
            .load_mut()
            .or(Err(ContractError::UnableToLoadAccountLoader))?,
        market_index,
        mark_price_before,
        now,
        valid_oracle_price,
    )?;

    if base_asset_amount == 0 {
        return Ok(0);
    }

    let mark_price_after: u128;
    let oracle_price_after: i128;
    let oracle_mark_spread_pct_after: i128;
    {
        let markets = &mut markets
            .load_mut()
            .or(Err(ContractError::UnableToLoadAccountLoader))?;
        let market = markets.get_market_mut(market_index);
        mark_price_after = market.amm.mark_price()?;
        let oracle_price_data = &market.amm.get_oracle_price(oracle, clock_slot)?;
        oracle_mark_spread_pct_after = amm::calculate_oracle_mark_spread_pct(
            &market.amm,
            oracle_price_data,
            Some(mark_price_after),
        )?;
        oracle_price_after = oracle_price_data.price;
    }

    let is_oracle_mark_too_divergent_before = amm::is_oracle_mark_too_divergent(
        oracle_mark_spread_pct_before,
        &state.oracle_guard_rails.price_divergence,
    )?;

    let is_oracle_mark_too_divergent_after = amm::is_oracle_mark_too_divergent(
        oracle_mark_spread_pct_after,
        &state.oracle_guard_rails.price_divergence,
    )?;

    // if oracle-mark divergence pushed outside limit, block order
    if is_oracle_mark_too_divergent_after && !is_oracle_mark_too_divergent_before && is_oracle_valid
    {
        return Err(ContractError::OracleMarkSpreadLimit);
    }

    // if oracle-mark divergence outside limit and risk-increasing, block order
    if is_oracle_mark_too_divergent_after
        && oracle_mark_spread_pct_after.unsigned_abs()
            >= oracle_mark_spread_pct_before.unsigned_abs()
        && is_oracle_valid
        && potentially_risk_increasing
    {
        return Err(ContractError::OracleMarkSpreadLimit);
    }

    // Order fails if it's risk increasing and it brings the user collateral below the margin requirement
    let meets_maintenance_requirement = if order.post_only {
        // for post only orders allow user to fill up to partial margin requirement
        meets_partial_margin_requirement(
            user,
            user_positions,
            &markets
                .load()
                .or(Err(ContractError::UnableToLoadAccountLoader))?,
        )?
    } else {
        meets_initial_margin_requirement(
            user,
            user_positions,
            &markets
                .load()
                .or(Err(ContractError::UnableToLoadAccountLoader))?,
        )?
    };
    if !meets_maintenance_requirement && potentially_risk_increasing {
        return Err(ContractError::InsufficientCollateral);
    }

    let discount_tier = order.discount_tier;
    let (user_fee, fee_to_market, token_discount, filler_reward, referrer_reward, referee_discount) =
        fees::calculate_fee_for_order(
            quote_asset_amount,
            &state.fee_structure,
            &order_state.order_filler_reward_structure,
            &discount_tier,
            order.ts,
            now,
            &referrer,
            filler.key() == user.key(),
            quote_asset_amount_surplus,
        )?;

    // Increment the clearing house's total fee variables
    {
        let markets = &mut markets
            .load_mut()
            .or(Err(ContractError::UnableToLoadAccountLoader))?;
        let market = markets.get_market_mut(market_index);
        market.amm.total_fee = market
            .amm
            .total_fee
            .checked_add(fee_to_market)
            .ok_or_else(|| (ContractError::MathError))?;
        market.amm.total_fee_minus_distributions = market
            .amm
            .total_fee_minus_distributions
            .checked_add(fee_to_market)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    // Subtract the fee from user's collateral
    user.collateral = user.collateral.checked_sub(user_fee).or(Some(0)).unwrap();

    // Increment the user's total fee variables
    user.total_fee_paid = user
        .total_fee_paid
        .checked_add(user_fee)
        .ok_or_else(|| (ContractError::MathError))?;
    user.total_token_discount = user
        .total_token_discount
        .checked_add(token_discount)
        .ok_or_else(|| (ContractError::MathError))?;
    user.total_referee_discount = user
        .total_referee_discount
        .checked_add(referee_discount)
        .ok_or_else(|| (ContractError::MathError))?;

    filler.collateral = filler
        .collateral
        .checked_add(cast(filler_reward)?)
        .ok_or_else(|| (ContractError::MathError))?;

    // Update the referrer's collateral with their reward
    if let Some(mut referrer) = referrer {
        referrer.total_referral_reward = referrer
            .total_referral_reward
            .checked_add(referrer_reward)
            .ok_or_else(|| (ContractError::MathError))?;
        referrer
            .exit(&crate::ID)
            .or(Err(ContractError::UnableToWriteToRemainingAccount))?;
    }

    {
        let markets = &mut markets
            .load()
            .or(Err(ContractError::UnableToLoadAccountLoader))?;
        let market = markets.get_market(market_index);
        update_order_after_trade(
            order,
            market.amm.minimum_base_asset_trade_size,
            base_asset_amount,
            quote_asset_amount,
            user_fee,
        )?;
    }

    let trade_history_account = &mut trade_history
        .load_mut()
        .or(Err(ContractError::UnableToLoadAccountLoader))?;
    let trade_record_id = trade_history_account.next_record_id();
    trade_history_account.append(TradeRecord {
        ts: now,
        record_id: trade_record_id,
        user_authority: user.authority,
        user: *user.to_account_info().key,
        direction: order.direction,
        base_asset_amount,
        quote_asset_amount,
        mark_price_before,
        mark_price_after,
        fee: user_fee,
        token_discount,
        referrer_reward,
        referee_discount,
        liquidation: false,
        market_index,
        oracle_price: oracle_price_after,
    });

    let order_history_account = &mut order_history
        .load_mut()
        .or(Err(ContractError::UnableToLoadAccountLoader))?;
    let record_id = order_history_account.next_record_id();
    order_history_account.append(OrderRecord {
        ts: now,
        record_id,
        order: *order,
        user: user.key(),
        authority: user.authority,
        action: OrderAction::Fill,
        filler: filler.key(),
        trade_record_id,
        base_asset_amount_filled: base_asset_amount,
        quote_asset_amount_filled: quote_asset_amount,
        filler_reward,
        fee: user_fee,
        quote_asset_amount_surplus,
        padding: [0; 8],
    });

    // Cant reset order until after its been logged in order history
    if order.base_asset_amount == order.base_asset_amount_filled
        || order.order_type == OrderType::Market
    {
        *order = Order::default();
        let position_index = get_position_index(user_positions, market_index)?;
        let market_position = &mut user_positions.positions[position_index];
        market_position.order_length -= 1;
    }

    // Try to update the funding rate at the end of every trade
    {
        let markets = &mut markets
            .load_mut()
            .or(Err(ContractError::UnableToLoadAccountLoader))?;
        let market = markets.get_market_mut(market_index);
        let funding_rate_history = &mut funding_rate_history
            .load_mut()
            .or(Err(ContractError::UnableToLoadAccountLoader))?;
        update_funding_rate(
            market_index,
            market,
            oracle,
            now,
            clock_slot,
            funding_rate_history,
            &state.oracle_guard_rails,
            state.funding_paused,
            Some(mark_price_before),
        )?;
    }

    Ok(base_asset_amount)
}

pub fn execute_order(
    deps: &DepsMut,
    user_addr: &Addr,
    order_index: u64,
    market_index: u64,
    mark_price_before: u128,
    now: u64,
    value_oracle_price: Option<i128>,
) -> Result<(u128, u128, bool, u128), ContractError> {
    match order.order_type {
        OrderType::Market => execute_market_order(
            deps,
            user,
            user_positions,
            order,
            markets,
            market_index,
            mark_price_before,
            now,
        ),
        _ => execute_non_market_order(
            user,
            user_positions,
            order,
            markets,
            market_index,
            mark_price_before,
            now,
            value_oracle_price,
        ),
    }
}

pub fn execute_market_order(
    deps: &DepsMut,
    user_addr: &Addr,
    order_index: u64,
    market_index: u64,
    mark_price_before: u128,
    now: u64,
) -> Result<(u128, u128, bool, u128), ContractError> {
    let position_index = get_position_index(user_positions, market_index)?;
    let market_position = &mut user_positions.positions[position_index];
    let market = markets.get_market_mut(market_index);

    let (potentially_risk_increasing, reduce_only, base_asset_amount, quote_asset_amount, _) =
        if order.base_asset_amount > 0 {
            controller::position::update_position_with_base_asset_amount(
                order.base_asset_amount,
                order.direction,
                market,
                user,
                market_position,
                mark_price_before,
                now,
                None,
            )?
        } else {
            controller::position::update_position_with_quote_asset_amount(
                order.quote_asset_amount,
                order.direction,
                market,
                user,
                market_position,
                mark_price_before,
                now,
            )?
        };

    if base_asset_amount < market.amm.minimum_base_asset_trade_size {
        msg!("base asset amount {}", base_asset_amount);
        return Err(ContractError::TradeSizeTooSmall);
    }

    if !reduce_only && order.reduce_only {
        return Err(ContractError::ReduceOnlyOrderIncreasedRisk);
    }

    if order.price > 0
        && !limit_price_satisfied(
            order.price,
            quote_asset_amount,
            base_asset_amount,
            order.direction,
        )?
    {
        return Err(ContractError::SlippageOutsideLimit);
    }

    Ok((
        base_asset_amount,
        quote_asset_amount,
        potentially_risk_increasing,
        0_u128,
    ))
}

pub fn execute_non_market_order(
    deps: &DepsMut,
    user_addr: &Addr,
    order_index: u64,
    market_index: u64,
    mark_price_before: u128,
    now: u64,
    valid_oracle_price: Option<i128>,
) -> Result<(u128, u128, bool, u128), ContractError> {
    // Determine the base asset amount the user can fill
    let base_asset_amount_user_can_execute = calculate_base_asset_amount_user_can_execute(
        deps,
        user_addr,
        order_index,
        market_index
    )?;

    if base_asset_amount_user_can_execute == 0 {
        // msg!("User cant execute order");
        return Ok((0, 0, false, 0));
    }

    let order = Orders.load(deps.storage, ((user_addr, market_index), order_index))?;
    let market = Markets.load(deps.storage, market_index)?;
    let market_position = Positions.load(deps.storage, (user_addr, market_index))?;

    // Determine the base asset amount the market can fill
    let base_asset_amount_market_can_execute = calculate_base_asset_amount_market_can_execute(
        &order,
        &market,
        Some(mark_price_before),
        valid_oracle_price,
    )?;

    if base_asset_amount_market_can_execute == 0 {
        // msg!("Market cant position_index : execute order");
        return Ok((0, 0, false, 0));
    }

    let mut base_asset_amount = min(
        base_asset_amount_market_can_execute,
        base_asset_amount_user_can_execute,
    );

    if base_asset_amount < market.amm.minimum_base_asset_trade_size {
        // msg!("base asset amount too small {}", base_asset_amount);
        return Ok((0, 0, false, 0));
    }

    let minimum_base_asset_trade_size = market.amm.minimum_base_asset_trade_size;
    let base_asset_amount_left_to_fill = order
        .base_asset_amount
        .checked_sub(
            order
                .base_asset_amount_filled
                .checked_add(base_asset_amount)
                .ok_or_else(|| (ContractError::MathError))?,
        )
        .ok_or_else(|| (ContractError::MathError))?;

    if base_asset_amount_left_to_fill > 0
        && base_asset_amount_left_to_fill < minimum_base_asset_trade_size
    {
        base_asset_amount = base_asset_amount
            .checked_add(base_asset_amount_left_to_fill)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    if base_asset_amount == 0 {
        return Ok((0, 0, false, 0));
    }

    let maker_limit_price = if order.post_only {
        Some(order.get_limit_price(valid_oracle_price)?)
    } else {
        None
    };
    let (
        potentially_risk_increasing,
        reduce_only,
        _,
        quote_asset_amount,
        quote_asset_amount_surplus,
    ) = update_position_with_base_asset_amount(
        deps,
        base_asset_amount,
        order.direction,
        user_addr,
        market_index,
        mark_price_before,
        now,
        maker_limit_price,
    )?;

    if !reduce_only && order.reduce_only {
        return Err(ContractError::ReduceOnlyOrderIncreasedRisk);
    }

    Ok((
        base_asset_amount,
        quote_asset_amount,
        potentially_risk_increasing,
        quote_asset_amount_surplus,
    ))
}

pub fn update_order_after_trade(
    deps: &DepsMut,
    user_addr: &Addr,
    position_index: u64,
    order_index: u64,
    minimum_base_asset_trade_size: u128,
    base_asset_amount: u128,
    quote_asset_amount: u128,
    fee: u128,
) -> Result<bool, ContractError>{
    let order = Orders.load(deps.storage, ((user_addr, position_index), order_index))?;
    order.base_asset_amount_filled = order
        .base_asset_amount_filled
        .checked_add(base_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    order.quote_asset_amount_filled = order
        .quote_asset_amount_filled
        .checked_add(quote_asset_amount)
        .ok_or_else(|| (ContractError::MathError))?;

    if order.order_type != OrderType::Market {
        // redundant test to make sure no min trade size remaining
        let base_asset_amount_to_fill = order
            .base_asset_amount
            .checked_sub(order.base_asset_amount_filled)
            .ok_or_else(|| (ContractError::MathError))?;

        if base_asset_amount_to_fill > 0
            && base_asset_amount_to_fill < minimum_base_asset_trade_size
        {
            return Err(ContractError::OrderAmountTooSmall);
        }
    }

    order.fee = order.fee.checked_add(fee).ok_or_else(|| (ContractError::MathError))?;

    Orders.update(deps.storage, ((user_addr, position_index), order_index), |o: Order| -> Result<Order, ContractError> {
        Ok(order)
    })?;

    Ok(true)
}
