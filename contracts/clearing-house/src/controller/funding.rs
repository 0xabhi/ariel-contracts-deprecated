use std::cmp::max;

use cosmwasm_std::Addr;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Uint128;

use crate::error::ContractError;

use crate::helpers::amm::normalise_oracle_price;
use crate::states::funding_history::{
    FUNDING_PAYMENT_HISTORY, FUNDING_PAYMENT_HISTORY_INFO, FundingPaymentInfo, FundingPaymentRecord,
    FUNDING_RATE_HISTORY, FUNDING_RATE_HISTORY_INFO, FundingRateInfo, FundingRateRecord,
};
use crate::states::market::{Market, MARKETS};
use crate::states::state::ORACLEGUARDRAILS;
use crate::states::user::{Position, POSITIONS, User, USERS};

use crate::helpers::collateral::calculate_updated_collateral;
use crate::helpers::constants::{
    AMM_TO_QUOTE_PRECISION_RATIO_I128, FUNDING_PAYMENT_PRECISION, ONE_HOUR,
};
use crate::helpers::funding::{calculate_funding_payment, calculate_funding_rate_long_short};
use crate::helpers::oracle;

use crate::controller::amm;

/// Funding payments are settled lazily. The amm tracks its cumulative funding rate (for longs and shorts)
/// and the user's market position tracks how much funding the user been cumulatively paid for that market.
/// If the two values are not equal, the user owes/is owed funding.
pub fn settle_funding_payment(
    deps: &mut DepsMut,
    user_addr: &Addr,
    now: u64,
) -> Result<(), ContractError> {
    let mut user = USERS.load(deps.storage, &user_addr.clone())?;
    let mut funding_payment: i128 = 0;

    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let mut market_position = POSITIONS.load(deps.storage, (user_addr, n))?;
            if market_position.base_asset_amount == 0 {
                continue;
            }
            let market = MARKETS.load(deps.storage, market_position.market_index)?;
            let amm_cumulative_funding_rate = if market_position.base_asset_amount > 0 {
                market.amm.cumulative_funding_rate_long
            } else {
                market.amm.cumulative_funding_rate_short
            };
            if amm_cumulative_funding_rate != market_position.last_cumulative_funding_rate {
                let market_funding_rate_payment =
                    calculate_funding_payment(amm_cumulative_funding_rate, &market_position)?;
                let funding_payment_history_info_length = FUNDING_PAYMENT_HISTORY_INFO
                    .load(deps.storage)?
                    .len
                    .checked_add(1)
                    .ok_or_else(|| (ContractError::MathError))?;
                FUNDING_PAYMENT_HISTORY_INFO.update(
                    deps.storage,
                    |mut i| -> Result<FundingPaymentInfo, ContractError> {
                        i.len = funding_payment_history_info_length;
                        Ok(i)
                    },
                )?;
                FUNDING_PAYMENT_HISTORY.save(
                    deps.storage,
                    (user_addr, funding_payment_history_info_length),
                    &FundingPaymentRecord {
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
                    },
                )?;
                funding_payment = funding_payment
                    .checked_add(market_funding_rate_payment)
                    .ok_or_else(|| (ContractError::MathError))?;

                market_position.last_cumulative_funding_rate = amm_cumulative_funding_rate;
                market_position.last_funding_rate_ts = market.amm.last_funding_rate_ts;

                POSITIONS.update(
                    deps.storage,
                    (user_addr, n),
                    |_p| -> Result<Position, ContractError> { Ok(market_position) },
                )?;
            }
        }
    }

    let funding_payment_collateral = funding_payment
        .checked_div(AMM_TO_QUOTE_PRECISION_RATIO_I128.u128() as i128)
        .ok_or_else(|| (ContractError::MathError))?;

    user.collateral = calculate_updated_collateral(user.collateral, funding_payment_collateral)?;

    USERS.update(
        deps.storage,
        user_addr,
        |_u| -> Result<User, ContractError> { Ok(user) },
    )?;

    Ok(())
}

pub fn update_funding_rate(
    deps: &mut DepsMut,
    market_index: u64,
    price_oracle: Addr,
    now: u64,
    funding_paused: bool,
    precomputed_mark_price: Option<Uint128>,
) -> Result<(), ContractError> {
    let mut market = MARKETS.load(deps.storage, market_index)?;
    let guard_rails = ORACLEGUARDRAILS.load(deps.storage)?;
    let price_oracle = market.amm.oracle.clone();

    let time_since_last_update = now
        .checked_sub(market.amm.last_funding_rate_ts)
        .ok_or_else(|| (ContractError::MathError))?;

    // Pause funding if oracle is invalid or if mark/oracle spread is too divergent
    let (block_funding_rate_update, oracle_price_data) = oracle::block_operation(
        &market.amm,
        &guard_rails,
        precomputed_mark_price,
    )?;

    let normalised_oracle_price =
        normalise_oracle_price(&market.amm, &oracle_price_data, precomputed_mark_price)?;

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
        let oracle_price_twap =
            amm::update_oracle_price_twap(deps, market_index, now, normalised_oracle_price)?;
        let mark_price_twap = amm::update_mark_twap(deps, market_index, now, None)?;

        let one_hour_i64 = ONE_HOUR.u128() as i64;
        let period_adjustment = (24_i64)
            .checked_mul(one_hour_i64)
            .ok_or_else(|| (ContractError::MathError))?
            .checked_div(max(one_hour_i64, market.amm.funding_period as i64))
            .ok_or_else(|| (ContractError::MathError))?;

        // funding period = 1 hour, window = 1 day
        // low periodicity => quickly updating/settled funding rates => lower funding rate payment per interval
        let price_spread = (mark_price_twap.u128()  as i128)
            .checked_sub(oracle_price_twap).ok_or_else(|| (ContractError::MathError))?;

        let funding_rate = price_spread
            .checked_mul(FUNDING_PAYMENT_PRECISION.u128() as i128)
            .ok_or_else(|| (ContractError::MathError))?
            .checked_div(period_adjustment as i128)
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

        MARKETS.update(
            deps.storage,
            market_index,
            |_m| -> Result<Market, ContractError> { Ok(market.clone()) },
        )?;

        let funding_rate_history_info_length = FUNDING_RATE_HISTORY_INFO
            .load(deps.storage)?
            .len
            .checked_add(1)
            .ok_or_else(|| (ContractError::MathError))?;
        FUNDING_RATE_HISTORY_INFO.update(
            deps.storage,
            |mut i: FundingRateInfo| -> Result<FundingRateInfo, ContractError> {
                i.len = funding_rate_history_info_length;
                Ok(i)
            },
        )?;
        FUNDING_RATE_HISTORY.save(
            deps.storage,
            funding_rate_history_info_length,
            &FundingRateRecord {
                ts: now,
                record_id: funding_rate_history_info_length,
                market_index,
                funding_rate,
                cumulative_funding_rate_long: market.amm.cumulative_funding_rate_long,
                cumulative_funding_rate_short: market.amm.cumulative_funding_rate_short,
                mark_price_twap,
                oracle_price_twap,
            },
        )?;
    };

    Ok(())
}
