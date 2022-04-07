use ariel::types::{OracleGuardRails, OracleStatus, OraclePriceData};
use cosmwasm_std::{Addr, DepsMut};

use crate::error::ContractError;
use crate::helpers::collateral::calculate_updated_collateral;
use crate::helpers::constants::MARGIN_PRECISION;
use crate::helpers::position::{
    calculate_base_asset_value_and_pnl, calculate_base_asset_value_and_pnl_with_oracle_price,
};
use crate::states::market::{Markets, LiquidationStatus, MarketStatus, LiquidationType};
use crate::states::user::{Users, Positions};

use crate::helpers::amm::use_oracle_price_for_margin_calculation;
use crate::helpers::casting::cast_to_i128;
use crate::helpers::oracle::{get_oracle_status};
use crate::helpers::slippage::calculate_slippage;

use std::ops::Div;

pub fn meets_initial_margin_requirement(
    deps: &DepsMut, 
    user_addr: &Addr,
) -> Result<bool, ContractError> {
    let user = Users.load(deps.storage, user_addr)?;

    let mut initial_margin_requirement: u128 = 0;
    let mut unrealized_pnl: i128 = 0;

    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let market_position = Positions.load(deps.storage, (user_addr, n))?;
            if market_position.base_asset_amount == 0 {
                continue;
            }
            let market = Markets.load(deps.storage, market_position.market_index)?;
            let a = &market.amm;
            let (position_base_asset_value, position_unrealized_pnl) =
                calculate_base_asset_value_and_pnl(&market_position, a)?;
            initial_margin_requirement = initial_margin_requirement
                .checked_add(
                    position_base_asset_value
                        .checked_mul(market.margin_ratio_initial.into())
                        .ok_or_else(|| (ContractError::HelpersError))?,
                )
                .ok_or_else(|| (ContractError::HelpersError))?;

            unrealized_pnl = unrealized_pnl
                .checked_add(position_unrealized_pnl)
                .ok_or_else(|| (ContractError::HelpersError))?;
        }
    }

    initial_margin_requirement = initial_margin_requirement
        .checked_div(MARGIN_PRECISION)
        .ok_or_else(|| (ContractError::HelpersError))?;

    let total_collateral = calculate_updated_collateral(user.collateral, unrealized_pnl)?;

    Ok(total_collateral >= initial_margin_requirement)
}

pub fn meets_partial_margin_requirement(
    deps: &DepsMut, 
    user_addr: &Addr,
) -> Result<bool, ContractError> {
    let user = Users.load(deps.storage, user_addr)?;

    let mut partial_margin_requirement: u128 = 0;
    let mut unrealized_pnl: i128 = 0;

    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let market_position = Positions.load(deps.storage, (user_addr, n))?;
            if market_position.base_asset_amount == 0 {
                continue;
            }
            let market = Markets.load(deps.storage, market_position.market_index)?;
            let a = &market.amm;
         
            let (position_base_asset_value, position_unrealized_pnl) =
            calculate_base_asset_value_and_pnl(&market_position, a)?;
            partial_margin_requirement = partial_margin_requirement
                .checked_add(
                    position_base_asset_value
                        .checked_mul(market.margin_ratio_partial.into())
                        .ok_or_else(|| (ContractError::HelpersError))?,
                )
                .ok_or_else(|| (ContractError::HelpersError))?;

            unrealized_pnl = unrealized_pnl
                .checked_add(position_unrealized_pnl)
                .ok_or_else(|| (ContractError::HelpersError))?;
            
        }
    }

    partial_margin_requirement = partial_margin_requirement
        .checked_div(MARGIN_PRECISION)
        .ok_or_else(|| (ContractError::HelpersError))?;

    let total_collateral = calculate_updated_collateral(user.collateral, unrealized_pnl)?;

    Ok(total_collateral >= partial_margin_requirement)
}

pub fn calculate_free_collateral(
    deps: &DepsMut, 
    user_addr: &Addr,
    market_to_close: Option<u64>,
) -> Result<(u128, u128), ContractError> {
    let mut closed_position_base_asset_value: u128 = 0;
    let mut initial_margin_requirement: u128 = 0;
    let mut unrealized_pnl: i128 = 0;

    let user = Users.load(deps.storage, user_addr)?;

    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let market_position = Positions.load(deps.storage, (user_addr, n))?;
            if market_position.base_asset_amount == 0 {
                continue;
            }

            let market = Markets.load(deps.storage, market_position.market_index)?;
            let a = &market.amm;
            let (position_base_asset_value, position_unrealized_pnl) =
                calculate_base_asset_value_and_pnl(&market_position, a)?;

            if market_to_close.is_some() && market_to_close.unwrap() == market_position.market_index {
                closed_position_base_asset_value = position_base_asset_value;
            } else {
                initial_margin_requirement = initial_margin_requirement
                    .checked_add(
                        position_base_asset_value
                            .checked_mul(market.margin_ratio_initial.into())
                            .ok_or_else(|| (ContractError::HelpersError))?,
                    )
                    .ok_or_else(|| (ContractError::HelpersError))?;
            }

            unrealized_pnl = unrealized_pnl
                .checked_add(position_unrealized_pnl)
                .ok_or_else(|| (ContractError::HelpersError))?;

        }
    }

    initial_margin_requirement = initial_margin_requirement
        .checked_div(MARGIN_PRECISION)
        .ok_or_else(|| (ContractError::HelpersError))?;

    let total_collateral = calculate_updated_collateral(user.collateral, unrealized_pnl)?;

    let free_collateral = if initial_margin_requirement < total_collateral {
        total_collateral
            .checked_sub(initial_margin_requirement)
            .ok_or_else(|| (ContractError::HelpersError))?
    } else {
        0
    };

    Ok((free_collateral, closed_position_base_asset_value))
}


pub fn calculate_liquidation_status(
    deps: &DepsMut, 
    user_addr: &Addr,
    oracle_guard_rails: &OracleGuardRails,
    oracle_account_info: &Addr,
    clock_slot: u64,
) -> Result<LiquidationStatus, ContractError> {

    let user = Users.load(deps.storage, user_addr)?;

    let mut partial_margin_requirement: u128 = 0;
    let mut maintenance_margin_requirement: u128 = 0;
    let mut base_asset_value: u128 = 0;
    let mut unrealized_pnl: i128 = 0;
    let mut adjusted_unrealized_pnl: i128 = 0;
    let mut market_statuses: Vec<MarketStatus> = Vec::new();

    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let market_position = Positions.load(deps.storage, (user_addr, n))?;
            if market_position.base_asset_amount == 0 {
                continue;
            }

            let market = Markets.load(deps.storage, market_position.market_index)?;
            let a = &market.amm;
            let (amm_position_base_asset_value, amm_position_unrealized_pnl) =
                calculate_base_asset_value_and_pnl(&market_position, a)?;

            base_asset_value = base_asset_value
                .checked_add(amm_position_base_asset_value)
                .ok_or_else(|| (ContractError::HelpersError))?;
            unrealized_pnl = unrealized_pnl
                .checked_add(amm_position_unrealized_pnl)
                .ok_or_else(|| (ContractError::HelpersError))?;

            // Block the liquidation if the oracle is invalid or the oracle and mark are too divergent
            let mark_price_before = market.amm.mark_price()?;

            let oracle_status = get_oracle_status(
                &market.amm,
                oracle_account_info,
                clock_slot,
                oracle_guard_rails,
                Some(mark_price_before),
            )?;

            let market_partial_margin_requirement: u128;
            let market_maintenance_margin_requirement: u128;
            let mut close_position_slippage = None;
            if oracle_status.is_valid
                && use_oracle_price_for_margin_calculation(
                    oracle_status.oracle_mark_spread_pct,
                    &oracle_guard_rails,
                )?
            {
                let market_index = market_position.market_index;
                let exit_slippage = calculate_slippage(
                    amm_position_base_asset_value,
                    market_position.base_asset_amount.unsigned_abs(),
                    cast_to_i128(mark_price_before)?,
                )?;
                close_position_slippage = Some(exit_slippage);

                let oracle_exit_price = oracle_status
                    .price_data
                    .price
                    .checked_add(exit_slippage)
                    .ok_or_else(|| (ContractError::HelpersError))?;

                let (oracle_position_base_asset_value, oracle_position_unrealized_pnl) =
                    calculate_base_asset_value_and_pnl_with_oracle_price(
                        &market_position,
                        oracle_exit_price,
                    )?;

                let oracle_provides_better_pnl =
                    oracle_position_unrealized_pnl > amm_position_unrealized_pnl;
                if oracle_provides_better_pnl {
                    adjusted_unrealized_pnl = adjusted_unrealized_pnl
                        .checked_add(oracle_position_unrealized_pnl)
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    market_partial_margin_requirement = (oracle_position_base_asset_value)
                        .checked_mul(market.margin_ratio_partial.into())
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    partial_margin_requirement = partial_margin_requirement
                        .checked_add(market_partial_margin_requirement)
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    market_maintenance_margin_requirement = oracle_position_base_asset_value
                        .checked_mul(market.margin_ratio_maintenance.into())
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    maintenance_margin_requirement = maintenance_margin_requirement
                        .checked_add(market_maintenance_margin_requirement)
                        .ok_or_else(|| (ContractError::HelpersError))?;
                } else {
                    adjusted_unrealized_pnl = adjusted_unrealized_pnl
                        .checked_add(amm_position_unrealized_pnl)
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    market_partial_margin_requirement = (amm_position_base_asset_value)
                        .checked_mul(market.margin_ratio_partial.into())
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    partial_margin_requirement = partial_margin_requirement
                        .checked_add(market_partial_margin_requirement)
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    market_maintenance_margin_requirement = amm_position_base_asset_value
                        .checked_mul(market.margin_ratio_maintenance.into())
                        .ok_or_else(|| (ContractError::HelpersError))?;

                    maintenance_margin_requirement = maintenance_margin_requirement
                        .checked_add(market_maintenance_margin_requirement)
                        .ok_or_else(|| (ContractError::HelpersError))?;
                }
            } else {
                adjusted_unrealized_pnl = adjusted_unrealized_pnl
                    .checked_add(amm_position_unrealized_pnl)
                    .ok_or_else(|| (ContractError::HelpersError))?;

                market_partial_margin_requirement = (amm_position_base_asset_value)
                    .checked_mul(market.margin_ratio_partial.into())
                    .ok_or_else(|| (ContractError::HelpersError))?;

                partial_margin_requirement = partial_margin_requirement
                    .checked_add(market_partial_margin_requirement)
                    .ok_or_else(|| (ContractError::HelpersError))?;

                market_maintenance_margin_requirement = amm_position_base_asset_value
                    .checked_mul(market.margin_ratio_maintenance.into())
                    .ok_or_else(|| (ContractError::HelpersError))?;

                maintenance_margin_requirement = maintenance_margin_requirement
                    .checked_add(market_maintenance_margin_requirement)
                    .ok_or_else(|| (ContractError::HelpersError))?;
            }

            market_statuses.push(MarketStatus {
                market_index: market_position.market_index,
                partial_margin_requirement: market_partial_margin_requirement.div(MARGIN_PRECISION),
                maintenance_margin_requirement: market_maintenance_margin_requirement
                    .div(MARGIN_PRECISION),
                base_asset_value: amm_position_base_asset_value,
                mark_price_before,
                oracle_status,
                close_position_slippage,
            });
        }
    }
    
    partial_margin_requirement = partial_margin_requirement
        .checked_div(MARGIN_PRECISION)
        .ok_or_else(|| (ContractError::HelpersError))?;

    maintenance_margin_requirement = maintenance_margin_requirement
        .checked_div(MARGIN_PRECISION)
        .ok_or_else(|| (ContractError::HelpersError))?;

    let total_collateral = calculate_updated_collateral(user.collateral, unrealized_pnl)?;
    let adjusted_total_collateral =
        calculate_updated_collateral(user.collateral, adjusted_unrealized_pnl)?;

    let requires_partial_liquidation = adjusted_total_collateral < partial_margin_requirement;
    let requires_full_liquidation = adjusted_total_collateral < maintenance_margin_requirement;

    let liquidation_type = if requires_full_liquidation {
        LiquidationType::FULL
    } else if requires_partial_liquidation {
        LiquidationType::PARTIAL
    } else {
        LiquidationType::NONE
    };

    let margin_requirement = match liquidation_type {
        LiquidationType::FULL => maintenance_margin_requirement,
        LiquidationType::PARTIAL => partial_margin_requirement,
        LiquidationType::NONE => partial_margin_requirement,
    };

    // Sort the market statuses such that we close the markets with biggest margin requirements first
    if liquidation_type == LiquidationType::FULL {
        market_statuses.sort_by(|a, b| {
            b.maintenance_margin_requirement
                .cmp(&a.maintenance_margin_requirement)
        });
    } else if liquidation_type == LiquidationType::PARTIAL {
        market_statuses.sort_by(|a, b| {
            b.partial_margin_requirement
                .cmp(&a.partial_margin_requirement)
        });
    }

    let margin_ratio = if base_asset_value == 0 {
        u128::MAX
    } else {
        total_collateral
            .checked_mul(MARGIN_PRECISION)
            .ok_or_else(|| (ContractError::HelpersError))?
            .checked_div(base_asset_value)
            .ok_or_else(|| (ContractError::HelpersError))?
    };

    Ok(LiquidationStatus {
        liquidation_type,
        margin_requirement,
        total_collateral,
        unrealized_pnl,
        adjusted_total_collateral,
        base_asset_value,
        market_statuses,
        margin_ratio,
    })
}
