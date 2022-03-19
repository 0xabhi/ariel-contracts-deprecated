use cosmwasm_std::Addr;

use crate::error::ContractError;

use crate::states::market::Market;
use crate::states::state::OracleGuardRails;

use crate::helpers::{amm};
use crate::helpers::constants::{
    SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_DENOMINATOR,
    SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_NUMERATOR,
};
use crate::helpers::position::_calculate_base_asset_value_and_pnl;
use crate::helpers::casting::cast_to_u128;
use crate::helpers::markets::get_oracle_price;

pub fn repeg(
    market: &mut Market,
    price_oracle: &Addr,
    new_peg: u128,
    clock_slot: u64,
    oracle_guard_rails: &OracleGuardRails,
) -> Result<i128, ContractError> {
    if new_peg == market.amm.peg_multiplier {
        return Err(ContractError::InvalidRepegRedundant.into());
    }

    let terminal_price_before = amm::calculate_terminal_price(market)?;

    let (current_net_market_value, _) =
        _calculate_base_asset_value_and_pnl(market.base_asset_amount, 0, &market.amm)?;

    market.amm.peg_multiplier = new_peg;

    let (_new_net_market_value, adjustment_cost) = _calculate_base_asset_value_and_pnl(
        market.base_asset_amount,
        current_net_market_value,
        &market.amm,
    )?;

    let (oracle_price, _oracle_twap, oracle_conf, _oracle_twac, _oracle_delay) =
        get_oracle_price(&market.amm, price_oracle, clock_slot)?;

    let oracle_is_valid = amm::is_oracle_valid(
        &market.amm,
        price_oracle,
        clock_slot,
        &oracle_guard_rails,
    )?;

    // if oracle is valid: check on size/direction of repeg
    if oracle_is_valid {
        let terminal_price_after = amm::calculate_terminal_price(market)?;

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
l 
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

    Ok(adjustment_cost)
}
