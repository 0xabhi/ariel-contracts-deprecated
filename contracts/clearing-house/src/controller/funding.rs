use std::cmp::{max, min};

use cosmwasm_std::{DepsMut};
use cosmwasm_std::Addr;

use crate::error::ContractError;

use crate::helpers::amm::normalise_oracle_price;
use crate::states::funding_history::{FundingPaymentHistory, FundingRateRecord, FundingPaymentRecord, FundingPaymentHistoryInfo, FundingRateInfo, FundingRateHistory, FundingRateHistoryInfo, FundingPaymentInfo};
use crate::states::market::{Markets, Market};
use crate::states::state::{STATE};
use crate::states::user::{Users, Positions, Position, User};

use crate::helpers::casting::{cast, cast_to_i128, cast_to_i64};
use crate::helpers::collateral::calculate_updated_collateral;
use crate::helpers::constants::{
    AMM_TO_QUOTE_PRECISION_RATIO_I128, FUNDING_PAYMENT_PRECISION, ONE_HOUR,
};
use crate::helpers::funding::{calculate_funding_payment, calculate_funding_rate_long_short};
use crate::helpers::{oracle};

use crate::controller::amm;

/// Funding payments are settled lazily. The amm tracks its cumulative funding rate (for longs and shorts)
/// and the user's market position tracks how much funding the user been cumulatively paid for that market.
/// If the two values are not equal, the user owes/is owed funding.
pub fn settle_funding_payment(
    deps: &mut DepsMut, 
    user_addr: &Addr,
    now: i64,
) -> Result<(), ContractError> {
    let mut user = Users.load(deps.storage, &user_addr.clone())?;
    let mut funding_payment: i128 = 0;

    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let mut market_position = Positions.load(deps.storage, (user_addr, n))?;
            if market_position.base_asset_amount == 0 {
                continue;
            }
            let market = Markets.load(deps.storage, market_position.market_index)?;
            let amm_cumulative_funding_rate = if market_position.base_asset_amount > 0 {
                market.amm.cumulative_funding_rate_long
            } else {
                market.amm.cumulative_funding_rate_short
            };
            if amm_cumulative_funding_rate != market_position.last_cumulative_funding_rate {
                let market_funding_rate_payment =
                calculate_funding_payment(amm_cumulative_funding_rate, &market_position)?;
                let funding_payment_history_info_length = 
                    FundingPaymentHistoryInfo.load(deps.storage)?
                    .len.checked_add(1).ok_or_else(|| (ContractError::MathError))?;
                FundingPaymentHistoryInfo.update(deps.storage, |mut i|-> Result<FundingPaymentInfo, ContractError> {
                    i.len = funding_payment_history_info_length;
                    Ok(i)
                })?;
                FundingPaymentHistory.save(deps.storage, (funding_payment_history_info_length, user_addr), &FundingPaymentRecord {
                    ts: now,
                    record_id: funding_payment_history_info_length,
                    user: user_addr.clone(),
                    market_index: market_position.market_index,
                    funding_payment: market_funding_rate_payment, //10e13
                    user_last_cumulative_funding: market_position.last_cumulative_funding_rate, //10e14
                    user_last_funding_rate_ts: market_position.last_funding_rate_ts,
                    amm_cumulative_funding_long: market.amm.cumulative_funding_rate_long, //10e14
                    amm_cumulative_funding_short: market.amm.cumulative_funding_rate_short, //10e14
                    base_asset_amount: market_position.base_asset_amount,     
                })?;
                funding_payment = funding_payment
                .checked_add(market_funding_rate_payment)
                .ok_or_else(|| (ContractError::MathError))?;

                market_position.last_cumulative_funding_rate = amm_cumulative_funding_rate;
                market_position.last_funding_rate_ts = market.amm.last_funding_rate_ts;
                
                Positions.update(deps.storage, (user_addr, n), |p| -> Result<Position, ContractError> {
                    Ok(market_position)
                })?;
            }
        }
    }

    let funding_payment_collateral = funding_payment
        .checked_div(AMM_TO_QUOTE_PRECISION_RATIO_I128)
        .ok_or_else(|| (ContractError::MathError))?;

    user.collateral = calculate_updated_collateral(user.collateral, funding_payment_collateral)?;

    Users.update(deps.storage, user_addr, |u|-> Result<User, ContractError> {
        Ok(user)
    })?;

    Ok(())
}

pub fn update_funding_rate(
    deps: &mut DepsMut,
    market_index: u64,
    price_oracle: Addr,
    now: i64,
    clock_slot: u64,
    funding_paused: bool,
    precomputed_mark_price: Option<u128>,
) -> Result<(), ContractError> {
    let mut market = Markets.load(deps.storage, market_index)?;
    let guard_rails = STATE.load(deps.storage)?.oracle_guard_rails;      
    
    let time_since_last_update = now
        .checked_sub(market.amm.last_funding_rate_ts)
        .ok_or_else(|| (ContractError::MathError))?;

    // Pause funding if oracle is invalid or if mark/oracle spread is too divergent
    let (block_funding_rate_update, oracle_price_data) =
        oracle::block_operation(  &market.amm,	
            &price_oracle,	
            clock_slot,	
            &guard_rails,	
            precomputed_mark_price
        )?;

    let normalised_oracle_price = normalise_oracle_price(&market.amm, &oracle_price_data, precomputed_mark_price)?;	

    // round next update time to be available on the hour
    let mut next_update_wait = market.amm.funding_period;
    if market.amm.funding_period > 1 {
        let last_update_delay = market
            .amm
            .last_funding_rate_ts
            .rem_euclid(market.amm.funding_period);
        if last_update_delay != 0 {
            let max_delay_for_next_period = market
                .amm
                .funding_period
                .checked_div(3)
                .ok_or_else(|| (ContractError::MathError))?;
            if last_update_delay > max_delay_for_next_period {
                // too late for on the hour next period, delay to following period
                next_update_wait = market
                    .amm
                    .funding_period
                    .checked_mul(2)
                    .ok_or_else(|| (ContractError::MathError))?
                    .checked_sub(last_update_delay)
                    .ok_or_else(|| (ContractError::MathError))?;
            } else {
                // allow update on the hour
                next_update_wait = market
                    .amm
                    .funding_period
                    .checked_sub(last_update_delay)
                    .ok_or_else(|| (ContractError::MathError))?;
            }
        }
    }

    if !funding_paused && !block_funding_rate_update && time_since_last_update >= next_update_wait {
        let oracle_price_twap = amm::update_oracle_price_twap(deps, market_index, now, normalised_oracle_price)?;
        let mark_price_twap = amm::update_mark_twap(deps, market_index, now, None)?;

        let one_hour_i64 = cast_to_i64(ONE_HOUR)?;
        let period_adjustment = (24_i64)
            .checked_mul(one_hour_i64)
            .ok_or_else(|| (ContractError::MathError))?
            .checked_div(max(one_hour_i64, market.amm.funding_period))
            .ok_or_else(|| (ContractError::MathError))?;
        
        // funding period = 1 hour, window = 1 day
        // low periodicity => quickly updating/settled funding rates => lower funding rate payment per interval
        let price_spread = cast_to_i128(mark_price_twap)?
            .checked_sub(oracle_price_twap)
            .ok_or_else(|| (ContractError::MathError))?;

        // clamp price divergence to 3% for funding rate calculation	
        let max_price_spread = oracle_price_twap	
        .checked_div(33)	
        .ok_or_else(|| (ContractError::MathError))?; // 3%	
        let clamped_price_spread = max(-max_price_spread, min(price_spread, max_price_spread));

        let funding_rate = price_spread
            .checked_mul(cast(FUNDING_PAYMENT_PRECISION)?)
            .ok_or_else(|| (ContractError::MathError))?
            .checked_div(cast(period_adjustment)?)
            .ok_or_else(|| (ContractError::MathError))?;

        let (funding_rate_long, funding_rate_short, new_total_fee_minus_distributions) =
            calculate_funding_rate_long_short(&market, funding_rate)?;

        market.amm.total_fee_minus_distributions = new_total_fee_minus_distributions;

        market.amm.cumulative_funding_rate_long = market
            .amm
            .cumulative_funding_rate_long
            .checked_add(funding_rate_long)
            .ok_or_else(|| (ContractError::MathError))?;

        market.amm.cumulative_funding_rate_short = market
            .amm
            .cumulative_funding_rate_short
            .checked_add(funding_rate_short)
            .ok_or_else(|| (ContractError::MathError))?;

        market.amm.last_funding_rate = funding_rate;
        market.amm.last_funding_rate_ts = now;

        Markets.update(deps.storage, market_index, |m| -> Result<Market, ContractError> {
            Ok(market.clone())
        })?;

        let funding_rate_history_info_length = 
            FundingRateHistoryInfo.load(deps.storage)?
            .len.checked_add(1).ok_or_else(|| (ContractError::MathError))?;
        FundingRateHistoryInfo.update(deps.storage, |mut i : FundingRateInfo |-> Result<FundingRateInfo, ContractError> {
            i.len = funding_rate_history_info_length;
            Ok(i)
        })?;
        FundingRateHistory.save(deps.storage, funding_rate_history_info_length, &FundingRateRecord {
            ts: now,
            record_id: funding_rate_history_info_length,
            market_index,
            funding_rate,
            cumulative_funding_rate_long: market.amm.cumulative_funding_rate_long,
            cumulative_funding_rate_short: market.amm.cumulative_funding_rate_short,
            mark_price_twap,
            oracle_price_twap,
        })?;
    };

    Ok(())
}
