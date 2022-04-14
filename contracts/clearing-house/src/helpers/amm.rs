use std::cmp::{max, min};

use crate::error::ContractError;

use ariel::types::{OracleGuardRails, SwapDirection, PositionDirection, OraclePriceData};

use crate::states::market::{Market, Amm};

use crate::helpers::bn::U192;
use crate::helpers::casting::{cast, cast_to_i128, cast_to_u128};
use crate::helpers::constants::{PEG_PRECISION, PRICE_TO_PEG_PRECISION_RATIO,MARK_PRICE_PRECISION, PRICE_SPREAD_PRECISION, PRICE_SPREAD_PRECISION_U128};
use crate::helpers::markets::{get_mark_price};
use crate::helpers::quote_asset::{reserve_to_asset_amount, asset_to_reserve_amount};

pub fn calculate_price(
    quote_asset_reserve: u128,
    base_asset_reserve: u128,
    peg_multiplier: u128,
) -> Result<u128, ContractError> {
    let peg_quote_asset_amount = quote_asset_reserve
        .checked_mul(peg_multiplier)
        .ok_or_else(|| (ContractError::MathError))?;

    U192::from(peg_quote_asset_amount)
        .checked_mul(U192::from(PRICE_TO_PEG_PRECISION_RATIO))
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(U192::from(base_asset_reserve))
        .ok_or_else(|| (ContractError::MathError))?
        .try_to_u128()
}

pub fn calculate_terminal_price(market: &mut Market) -> Result<u128, ContractError> {
    let swap_direction = if market.base_asset_amount > 0 {
        SwapDirection::Add
    } else {
        SwapDirection::Remove
    };
    let (new_quote_asset_amount, new_base_asset_amount) = calculate_swap_output(
        market.base_asset_amount.unsigned_abs(),
        market.amm.base_asset_reserve,
        swap_direction,
        market.amm.sqrt_k,
    )?;

    let terminal_price = calculate_price(
        new_quote_asset_amount,
        new_base_asset_amount,
        market.amm.peg_multiplier,
    )?;

    Ok(terminal_price)
}

pub fn calculate_new_mark_twap(
    a: &Amm,
    now: u64,
    precomputed_mark_price: Option<u128>,
) -> Result<u128, ContractError> {
    let since_last = cast_to_i128(max(
        1,
        now.checked_sub(a.last_mark_price_twap_ts)
            .ok_or_else(|| (ContractError::MathError))?,
    ))?;
    let from_start = max(
        1,
        cast_to_i128(a.funding_period)?
            .checked_sub(since_last)
            .ok_or_else(|| (ContractError::MathError))?,
    );
    let current_price = match precomputed_mark_price {
        Some(mark_price) => mark_price,
        None => get_mark_price(&a)?,
    };

    let new_twap: u128 = cast(calculate_twap(
        cast(current_price)?,
        cast(a.last_mark_price_twap)?,
        since_last,
        from_start,
    )?)?;

    return Ok(new_twap);
}

pub fn calculate_new_oracle_price_twap(
    a: &Amm,
    now: u64,
    oracle_price: i128,
) -> Result<i128, ContractError> {
    let since_last = cast_to_i128(max(
        1,
        now.checked_sub(a.last_oracle_price_twap_ts)
            .ok_or_else(|| (ContractError::MathError))?,
    ))?;
    let from_start = max(
        1,
        cast_to_i128(a.funding_period)?
            .checked_sub(since_last)
            .ok_or_else(|| (ContractError::MathError))?,
    );

    // ensure amm.last_oracle_price is proper
    // let capped_last_oracle_price = if a.last_oracle_price > 0 {
    //     a.last_oracle_price
    // } else {
    //     oracle_price
    // };

    // nudge last_oracle_price up to .1% toward oracle price
    // let capped_last_oracle_price_10bp = capped_last_oracle_price
    // .checked_div(1000)
    // .ok_or_else(|| (ContractError::MathError))?;

    // let interpolated_oracle_price = min(
    //     capped_last_oracle_price
    //         .checked_add(capped_last_oracle_price_10bp)
    //         .ok_or_else(|| (ContractError::MathError))?,
    //     max(
    //         capped_last_oracle_price
    //             .checked_sub(capped_last_oracle_price_10bp)
    //             .ok_or_else(|| (ContractError::MathError))?,
    //         oracle_price,
    //     ),
    // );

    let new_twap = calculate_twap(
        oracle_price,
        a.last_oracle_price_twap,
        since_last,
        from_start,
    )?;

    return Ok(new_twap);
}

pub fn calculate_twap(
    new_data: i128,
    old_data: i128,
    new_weight: i128,
    old_weight: i128,
) -> Result<i128, ContractError> {
    let denominator = new_weight
        .checked_add(old_weight)
        .ok_or_else(|| (ContractError::MathError))?;
    let prev_twap_99 = old_data.checked_mul(old_weight).ok_or_else(|| (ContractError::MathError))?;
    let latest_price_01 = new_data.checked_mul(new_weight).ok_or_else(|| (ContractError::MathError))?;
    let new_twap = prev_twap_99
        .checked_add(latest_price_01)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(denominator)
        .ok_or_else(|| (ContractError::MathError));
    return new_twap;
}

pub fn calculate_swap_output(
    swap_amount: u128,
    input_asset_amount: u128,
    direction: SwapDirection,
    invariant_sqrt: u128,
) -> Result<(u128, u128), ContractError> {
    let invariant_sqrt_u192 = U192::from(invariant_sqrt);
    let invariant = invariant_sqrt_u192
        .checked_mul(invariant_sqrt_u192)
        .ok_or_else(|| (ContractError::MathError))?;

    if direction == SwapDirection::Remove && swap_amount > input_asset_amount {
        return Err(ContractError::TradeSizeTooLarge);
    }

    let new_input_amount = if let SwapDirection::Add = direction {
        input_asset_amount
            .checked_add(swap_amount)
            .ok_or_else(|| (ContractError::MathError))?
    } else {
        input_asset_amount
            .checked_sub(swap_amount)
            .ok_or_else(|| (ContractError::MathError))?
    };

    let new_input_amount_u192 = U192::from(new_input_amount);
    let new_output_amount = invariant
        .checked_div(new_input_amount_u192)
        .ok_or_else(|| (ContractError::MathError))?
        .try_to_u128()?;

    return Ok((new_output_amount, new_input_amount));
}

pub fn calculate_quote_asset_amount_swapped(
    quote_asset_reserve_before: u128,
    quote_asset_reserve_after: u128,
    swap_direction: SwapDirection,
    peg_multiplier: u128,
) -> Result<u128, ContractError> {
    let quote_asset_reserve_change = match swap_direction {
        SwapDirection::Add => quote_asset_reserve_before
            .checked_sub(quote_asset_reserve_after)
            .ok_or_else(|| (ContractError::MathError))?,

        SwapDirection::Remove => quote_asset_reserve_after
            .checked_sub(quote_asset_reserve_before)
            .ok_or_else(|| (ContractError::MathError))?,
    };

    let mut quote_asset_amount =
    reserve_to_asset_amount(quote_asset_reserve_change, peg_multiplier)?;

    // when a user goes long base asset, make the base asset slightly more expensive
    // by adding one unit of quote asset
    if swap_direction == SwapDirection::Remove {
        quote_asset_amount = quote_asset_amount
            .checked_add(1)
            .ok_or_else(|| (ContractError::MathError))?;
    }

    Ok(quote_asset_amount)
}


pub fn normalise_oracle_price(
    a: &Amm,
    oracle_price: &OraclePriceData,
    precomputed_mark_price: Option<u128>,
) -> Result<i128, ContractError> {
    let OraclePriceData {
        price: oracle_price,
        confidence: oracle_conf,
        ..
    } = *oracle_price;

    let mark_price = match precomputed_mark_price {
        Some(mark_price) => cast_to_i128(mark_price)?,
        None => cast_to_i128(a.mark_price()?)?,
    };

    let mark_price_1bp = mark_price.checked_div(10000).ok_or_else(|| (ContractError::MathError))?;
    let conf_int = cast_to_i128(oracle_conf)?;

    //  normalises oracle toward mark price based on the oracle’s confidence interval
    //  if mark above oracle: use oracle+conf unless it exceeds .9999 * mark price
    //  if mark below oracle: use oracle-conf unless it less than 1.0001 * mark price
    //  (this guarantees more reasonable funding rates in volatile periods)
    let normalised_price = if mark_price > oracle_price {
        min(
            max(
                mark_price
                    .checked_sub(mark_price_1bp)
                    .ok_or_else(|| (ContractError::MathError))?,
                oracle_price,
            ),
            oracle_price
                .checked_add(conf_int)
                .ok_or_else(|| (ContractError::MathError))?,
        )
    } else {
        max(
            min(
                mark_price
                    .checked_add(mark_price_1bp)
                    .ok_or_else(|| (ContractError::MathError))?,
                oracle_price,
            ),
            oracle_price
                .checked_sub(conf_int)
                .ok_or_else(|| (ContractError::MathError))?,
        )
    };

    Ok(normalised_price)
}


pub fn calculate_oracle_mark_spread(
    a: &Amm,
    oracle_price_data: &OraclePriceData,
    precomputed_mark_price: Option<u128>,
) -> Result<(i128, i128), ContractError> {
    let mark_price = match precomputed_mark_price {
        Some(mark_price) => cast_to_i128(mark_price)?,
        None => cast_to_i128(a.mark_price()?)?,
    };

    let oracle_price = oracle_price_data.price;

    let price_spread = mark_price
        .checked_sub(oracle_price)
        .ok_or_else(|| (ContractError::MathError))?;

    Ok((oracle_price, price_spread))

}

pub fn calculate_oracle_mark_spread_pct(
    a: &Amm,
    oracle_price_data: &OraclePriceData,
    precomputed_mark_price: Option<u128>,
) -> Result<i128, ContractError> {
    let (oracle_price, price_spread) =
        calculate_oracle_mark_spread(a, oracle_price_data, precomputed_mark_price)?;

    price_spread
        .checked_mul(PRICE_SPREAD_PRECISION)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(oracle_price)
        .ok_or_else(|| (ContractError::MathError))
}

pub fn is_oracle_mark_too_divergent(
    price_spread_pct: i128,
    oracle_guard_rails: &OracleGuardRails,
) -> Result<bool, ContractError> {
    let max_divergence = oracle_guard_rails
        .mark_oracle_divergence_numerator
        .checked_shl(10)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(oracle_guard_rails.mark_oracle_divergence_denominator)
        .ok_or_else(|| (ContractError::MathError))?;

    Ok(price_spread_pct.unsigned_abs() > max_divergence)
}

pub fn calculate_mark_twap_spread_pct(a: &Amm, mark_price: u128) -> Result<i128, ContractError> {
    let mark_price = cast_to_i128(mark_price)?;
    let mark_twap = cast_to_i128(a.last_mark_price_twap)?;

    let price_spread = mark_price
        .checked_sub(mark_twap)
        .ok_or_else(|| (ContractError::MathError))?;

    price_spread
        .checked_mul(PRICE_SPREAD_PRECISION)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(mark_twap)
        .ok_or_else(|| (ContractError::MathError))
}

pub fn use_oracle_price_for_margin_calculation(
    price_spread_pct: i128,
    oracle_guard_rails: &OracleGuardRails,
) -> Result<bool, ContractError> {
    let max_divergence = oracle_guard_rails
        .mark_oracle_divergence_numerator
        .checked_mul(PRICE_SPREAD_PRECISION_U128)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(oracle_guard_rails.mark_oracle_divergence_denominator)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(3)
        .ok_or_else(|| (ContractError::MathError))?;

    Ok(price_spread_pct.unsigned_abs() > max_divergence)
}


pub fn is_oracle_valid(
    a: &Amm,
    oracle_price_data: &OraclePriceData,
    valid_oracle_guard_rails: &OracleGuardRails,
) -> Result<bool, ContractError> {
    let OraclePriceData {
        price: oracle_price,
        confidence: oracle_conf,
        delay: oracle_delay,
        has_sufficient_number_of_data_points,
        ..
    } = *oracle_price_data;

    let is_oracle_price_nonpositive = oracle_price <= 0;

    let is_oracle_price_too_volatile = ((oracle_price
        .checked_div(max(1, a.last_oracle_price_twap))
        .ok_or_else(|| (ContractError::MathError))?)
    .gt(&valid_oracle_guard_rails.too_volatile_ratio))
        || ((a
            .last_oracle_price_twap
            .checked_div(max(1, oracle_price))
            .ok_or_else(|| (ContractError::MathError))?)
        .gt(&valid_oracle_guard_rails.too_volatile_ratio));

    let conf_denom_of_price = cast_to_u128(oracle_price)?
        .checked_div(max(1, oracle_conf))
        .ok_or_else(|| (ContractError::MathError))?;
    let is_conf_too_large =
        conf_denom_of_price.lt(&valid_oracle_guard_rails.confidence_interval_max_size);

    let is_stale = oracle_delay.gt(&valid_oracle_guard_rails.slots_before_stale);

    Ok(!(is_stale
        || !has_sufficient_number_of_data_points
        || is_oracle_price_nonpositive
        || is_oracle_price_too_volatile
        || is_conf_too_large))
}

pub fn calculate_max_base_asset_amount_to_trade(
    amm: &Amm,
    limit_price: u128,
) -> Result<(u128, PositionDirection), ContractError> {
    let invariant_sqrt_u192 = U192::from(amm.sqrt_k);
    let invariant = invariant_sqrt_u192
        .checked_mul(invariant_sqrt_u192)
        .ok_or_else(|| (ContractError::MathError))?;

    let new_base_asset_reserve_squared = invariant
        .checked_mul(U192::from(MARK_PRICE_PRECISION))
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(U192::from(limit_price))
        .ok_or_else(|| (ContractError::MathError))?
        .checked_mul(U192::from(amm.peg_multiplier))
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(U192::from(PEG_PRECISION))
        .ok_or_else(|| (ContractError::MathError))?;

    let new_base_asset_reserve = new_base_asset_reserve_squared
        .integer_sqrt()
        .try_to_u128()?;

    if new_base_asset_reserve > amm.base_asset_reserve {
        let max_trade_amount = new_base_asset_reserve
            .checked_sub(amm.base_asset_reserve)
            .ok_or_else(|| (ContractError::MathError))?;
        Ok((max_trade_amount, PositionDirection::Short))
    } else {
        let max_trade_amount = amm
            .base_asset_reserve
            .checked_sub(new_base_asset_reserve)
            .ok_or_else(|| (ContractError::MathError))?;
        Ok((max_trade_amount, PositionDirection::Long))
    }
}

pub fn should_round_trade(
    a: &Amm,
    quote_asset_amount: u128,
    base_asset_value: u128,
) -> Result<bool, ContractError> {
    let difference = if quote_asset_amount > base_asset_value {
        quote_asset_amount
            .checked_sub(base_asset_value)
            .ok_or_else(|| (ContractError::MathError))?
    } else {
        base_asset_value
            .checked_sub(quote_asset_amount)
            .ok_or_else(|| (ContractError::MathError))?
    };

    let quote_asset_reserve_amount = asset_to_reserve_amount(difference, a.peg_multiplier)?;

    Ok(quote_asset_reserve_amount < a.minimum_quote_asset_trade_size)
}

