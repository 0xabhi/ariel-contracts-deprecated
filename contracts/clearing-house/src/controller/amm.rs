use cosmwasm_std::{DepsMut};

use crate::error::{ContractError};

use crate::states::state::SwapDirection;
use crate::states::market::{Market, Markets};

use crate::helpers::amm::calculate_quote_asset_amount_swapped;
use crate::helpers::casting::{cast, cast_to_i128};
use crate::helpers::constants::{MARK_PRICE_PRECISION};
use crate::helpers::{amm, bn};
use crate::helpers::position::_calculate_base_asset_value_and_pnl;
use crate::helpers::quote_asset::{asset_to_reserve_amount};

pub fn update_mark_twap(
    deps: DepsMut,
    market_index: u64,
    now: i64,
    precomputed_mark_price: Option<u128>,
) -> Result<u128, ContractError> {
    let market = Markets.load(deps.storage, market_index)?;
    let mark_twap = amm::calculate_new_mark_twap(&market.amm, now, precomputed_mark_price)?;
    Markets.update(deps.storage, market_index, |m| -> Result<Market, ContractError> {
        match m {
            Some(mark) => {
                mark.amm.last_mark_price_twap = mark_twap;
                mark.amm.last_mark_price_twap_ts = now;
                Ok(mark)
            },
            None => {
                Err(ContractError::MarketIndexNotInitialized{})
            }
        }
    })?;
    return Ok(mark_twap);
}

pub fn update_oracle_price_twap(
    deps: DepsMut,
    market_index: u64,
    now: i64,
    oracle_price: i128,
) -> Result<i128, ContractError> {
    let market = Markets.load(deps.storage, market_index)?;
    let oracle_price_twap = amm::calculate_new_oracle_price_twap(&market.amm, now, oracle_price)?;
    Markets.update(deps.storage, market_index, |m| -> Result<Market, ContractError> {
        match m {
            Some(mark) => {
                mark.amm.last_oracle_price_twap = oracle_price_twap;
                mark.amm.last_oracle_price_twap_ts = now;
                Ok(mark)
            },
            None => {
                Err(ContractError::MarketIndexNotInitialized{})
            }
        }
    })?;
 
    return Ok(oracle_price_twap);
}

/// To find the cost of adjusting k, compare the the net market value before and after adjusting k
/// Increasing k costs the protocol money because it reduces slippage and improves the exit price for net market position
/// Decreasing k costs the protocol money because it increases slippage and hurts the exit price for net market position
pub fn adjust_k_cost(deps: DepsMut, market_index: u64, new_sqrt_k: bn::U256) -> Result<i128, ContractError> {
    let market = Markets.load(deps.storage, market_index)?;
    // Find the net market value before adjusting k
    let (current_net_market_value, _) =
        _calculate_base_asset_value_and_pnl(market.base_asset_amount, 0, &market.amm)?;

    let ratio_scalar = bn::U256::from(MARK_PRICE_PRECISION);

    let sqrt_k_ratio = new_sqrt_k
        .checked_mul(ratio_scalar)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(bn::U256::from(market.amm.sqrt_k))
        .ok_or_else(|| (ContractError::MathError))?;

    // if decreasing k, max decrease ratio for single transaction is 2.5%
    if sqrt_k_ratio
        < ratio_scalar
            .checked_mul(bn::U256::from(975))
            .ok_or_else(|| (ContractError::MathError))?
            .checked_div(bn::U256::from(1000))
            .ok_or_else(|| (ContractError::MathError))?
    {
        return Err(ContractError::InvalidUpdateK.into());
    }
    let new_sqrt_k_val= new_sqrt_k.try_to_u128().unwrap();
    let new_base_asset_reserve = bn::U256::from(market.amm.base_asset_reserve)
        .checked_mul(sqrt_k_ratio)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(ratio_scalar)
        .ok_or_else(|| (ContractError::MathError))?
        .try_to_u128()
        .unwrap();
    let new_quote_asset_reserve = bn::U256::from(market.amm.quote_asset_reserve)
        .checked_mul(sqrt_k_ratio)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(ratio_scalar)
        .ok_or_else(|| (ContractError::MathError))?
        .try_to_u128()
        .unwrap();

    Markets.update(deps.storage, market_index, |m| -> Result<Market, ContractError> {
        match m {
            Some(mark) => {
                mark.amm.sqrt_k = new_sqrt_k_val;
                mark.amm.base_asset_reserve = new_base_asset_reserve;
                mark.amm.quote_asset_reserve = new_quote_asset_reserve;
                Ok(mark)
            },
            None => {
                Err(ContractError::MarketIndexNotInitialized{})
            }
        }
    })?;

    let (_new_net_market_value, cost) = _calculate_base_asset_value_and_pnl(
        market.base_asset_amount,
        current_net_market_value,
        &market.amm,
    )?;

    Ok(cost)
}

pub fn swap_quote_asset(
    deps: DepsMut,
    market_index: u64,
    quote_asset_amount: u128,
    direction: SwapDirection,
    now: i64,
    precomputed_mark_price: Option<u128>,
) -> Result<i128, ContractError> {
    let market = Markets.load(deps.storage, market_index)?;
    let a = market.amm;
    update_mark_twap(deps, market_index, now, precomputed_mark_price)?;
    let quote_asset_reserve_amount =
        asset_to_reserve_amount(quote_asset_amount, a.peg_multiplier)?;

    if quote_asset_reserve_amount < a.minimum_trade_size {
        return Err(ContractError::TradeSizeTooSmall);
    }

    let initial_base_asset_reserve = a.base_asset_reserve;
    let (new_base_asset_reserve, new_quote_asset_reserve) = amm::calculate_swap_output(
        quote_asset_reserve_amount,
        a.quote_asset_reserve,
        direction,
        a.sqrt_k,
    )?;

    Markets.update(deps.storage, market_index, |m| -> Result<Market, ContractError> {
        match m {
            Some(mark) => {
                mark.amm.base_asset_reserve = new_base_asset_reserve;
                mark.amm.quote_asset_reserve = new_quote_asset_reserve;
                Ok(mark)
            },
            None => {
                Err(ContractError::MarketIndexNotInitialized{})
            }
        }
    })?;

    let base_asset_amount = cast_to_i128(initial_base_asset_reserve)?
        .checked_sub(cast(new_base_asset_reserve)?)
        .ok_or_else(|| (ContractError::MathError))?;

    return Ok(base_asset_amount);
}

pub fn swap_base_asset(
    deps: DepsMut,
    market_index: u64,
    base_asset_swap_amount: u128,
    direction: SwapDirection,
    now: i64,
) -> Result<u128, ContractError> {
    let market = Markets.load(deps.storage, market_index)?;
    let a = market.amm;
    
    update_mark_twap(deps, market_index, now, None)?;

    let initial_quote_asset_reserve = a.quote_asset_reserve;
    let (new_quote_asset_reserve, new_base_asset_reserve) = amm::calculate_swap_output(
        base_asset_swap_amount,
        a.base_asset_reserve,
        direction,
        a.sqrt_k,
    )?;

    Markets.update(deps.storage, market_index, |m| -> Result<Market, ContractError> {
        match m {
            Some(mark) => {
                mark.amm.base_asset_reserve = new_base_asset_reserve;
                mark.amm.quote_asset_reserve = new_quote_asset_reserve;
                Ok(mark)
            },
            None => {
                Err(ContractError::MarketIndexNotInitialized{})
            }
        }
    })?;

    calculate_quote_asset_amount_swapped(
        initial_quote_asset_reserve,
        new_quote_asset_reserve,
        direction,
        a.peg_multiplier,
    )
}

pub fn move_price(
    deps: DepsMut, 
    market_index: u64,
    base_asset_reserve: u128,
    quote_asset_reserve: u128,
) -> Result<(), ContractError> {
    let k = bn::U256::from(base_asset_reserve)
        .checked_mul(bn::U256::from(quote_asset_reserve))
        .ok_or_else(|| (ContractError::MathError))?;

    Markets.update(deps.storage, market_index, |m| -> Result<Market, ContractError> {
        match m {
            Some(mark) => {
                mark.amm.base_asset_reserve = base_asset_reserve;
                mark.amm.quote_asset_reserve = quote_asset_reserve;
                mark.amm.sqrt_k = k.integer_sqrt().try_to_u128()?;
                Ok(mark)
            },
            None => {
                Err(ContractError::MarketIndexNotInitialized{})
            }
        }
    })?;
    Ok(())
}
