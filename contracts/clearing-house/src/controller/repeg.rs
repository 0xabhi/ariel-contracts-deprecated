use cosmwasm_std::{Addr, DepsMut};

use crate::error::ContractError;

use crate::states::market::{Markets, Market};
use crate::states::state::{STATE};

use crate::helpers::{amm};
use crate::helpers::constants::{
    SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_DENOMINATOR,
    SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_NUMERATOR,
};
use crate::helpers::position::_calculate_base_asset_value_and_pnl;
use crate::helpers::casting::cast_to_u128;
use crate::helpers::oracle::get_oracle_price;

pub fn repeg(
    deps: &mut DepsMut,
    market_index: u64,
    price_oracle: &Addr,
    new_peg_candidate: u128
) -> Result<i128, ContractError> {

    let mut market = Markets.load(deps.storage, market_index)?;

    let state = STATE.load(deps.storage)?;
    let oracle_guard_rails = state.oracle_guard_rails;

    if new_peg_candidate == market.amm.peg_multiplier {
        return Err(ContractError::InvalidRepegRedundant.into());
    }

    let terminal_price_before = amm::calculate_terminal_price(&mut market)?;
    let adjustment_cost = adjust_peg_cost(&mut market, new_peg_candidate)?;

    let (current_net_market_value, _) =
        _calculate_base_asset_value_and_pnl(market.base_asset_amount, 0, &market.amm)?;

    market.amm.peg_multiplier = new_peg_candidate;

    let oracle_price_data = get_oracle_price(&market.amm, price_oracle)?;	
    let oracle_price = oracle_price_data.price;	
    let oracle_conf = oracle_price_data.confidence;	
    let oracle_is_valid =	
        amm::is_oracle_valid(&market.amm, &oracle_price_data, &oracle_guard_rails)?;	
    
    // if oracle is valid: check on size/direction of repeg
    if oracle_is_valid {
        let terminal_price_after = amm::calculate_terminal_price(&mut market)?;

        let mark_price_after = amm::calculate_price(
            market.amm.quote_asset_reserve,
            market.amm.base_asset_reserve,
            market.amm.peg_multiplier,
        )?;

        let oracle_conf_band_top = cast_to_u128(oracle_price)?
            .checked_add(oracle_conf)
            .ok_or_else(|| (ContractError::MathError))?;

        let oracle_conf_band_bottom = cast_to_u128(oracle_price)?
            .checked_sub(oracle_conf)
            .ok_or_else(|| (ContractError::MathError))?;

        if cast_to_u128(oracle_price)? > terminal_price_after {
            // only allow terminal up when oracle is higher
            if terminal_price_after < terminal_price_before {
                return Err(ContractError::InvalidRepegDirection.into());
            }

            // only push terminal up to top of oracle confidence band
            if oracle_conf_band_bottom < terminal_price_after {
                return Err(ContractError::InvalidRepegProfitability.into());
            }

            // only push mark up to top of oracle confidence band
            if mark_price_after > oracle_conf_band_top {
                return Err(ContractError::InvalidRepegProfitability.into());
            }
        }

        if cast_to_u128(oracle_price)? < terminal_price_after {
            // only allow terminal down when oracle is lower
            if terminal_price_after > terminal_price_before {
                return Err(ContractError::InvalidRepegDirection.into());
            }

            // only push terminal down to top of oracle confidence band
            if oracle_conf_band_top > terminal_price_after {
                return Err(ContractError::InvalidRepegProfitability.into());
            }

            // only push mark down to bottom of oracle confidence band
            if mark_price_after < oracle_conf_band_bottom {
                return Err(ContractError::InvalidRepegProfitability.into());
            }
        }
    }

    // Reduce pnl to quote asset precision and take the absolute value
    if adjustment_cost > 0 {
        market.amm.total_fee_minus_distributions = market
            .amm
            .total_fee_minus_distributions
            .checked_sub(adjustment_cost.unsigned_abs())
            .or(Some(0))
            .ok_or_else(|| (ContractError::MathError))?;

        // Only a portion of the protocol fees are allocated to repegging
        // This checks that the total_fee_minus_distributions does not decrease too much after repeg
        if market.amm.total_fee_minus_distributions
            < market
                .amm
                .total_fee
                .checked_mul(SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_NUMERATOR)
                .ok_or_else(|| (ContractError::MathError))?
                .checked_div(SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_DENOMINATOR)
                .ok_or_else(|| (ContractError::MathError))?
        {
            return Err(ContractError::InvalidRepegProfitability.into());
        }
    } else {
        market.amm.total_fee_minus_distributions = market
            .amm
            .total_fee_minus_distributions
            .checked_add(adjustment_cost.unsigned_abs())
            .ok_or_else(|| (ContractError::MathError))?;
    }

    Markets.update(deps.storage, market_index, |m| ->  Result<Market, ContractError>{
        Ok(market)
    });

    Ok(adjustment_cost)

}

pub fn adjust_peg_cost(market: &mut Market, new_peg: u128) -> Result<i128, ContractError> {
    // Find the net market value before adjusting peg
    let (current_net_market_value, _) =
        _calculate_base_asset_value_and_pnl(market.base_asset_amount, 0, &market.amm)?;

    market.amm.peg_multiplier = new_peg;

    let (_new_net_market_value, cost) = _calculate_base_asset_value_and_pnl(
        market.base_asset_amount,
        current_net_market_value,
        &market.amm,
    )?;

    Ok(cost)
}
