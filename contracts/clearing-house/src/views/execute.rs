use crate::controller;
use crate::helpers;
use crate::ContractError;
// use crate::helpers::casting::cast_to_i64;
use crate::helpers::casting::*;
use crate::helpers::constants::*;
use crate::helpers::withdrawal::calculate_withdrawal_amounts;
use crate::states::curve_history::*;

use crate::states::deposit_history::*;
use crate::states::market::{Amm, Market, Markets};
use crate::states::state::ADMIN;
use crate::states::state::{State, STATE};
use crate::states::user::{is_for, Position, Positions, User, Users};

use ariel::helper::addr_validate_to_lower;
use ariel::helper::assert_sent_uusd_balance;
use ariel::helper::query_balance;
use ariel::helper::VaultInterface;
use ariel::types::Order;
use ariel::types::{
    DepositDirection, DiscountTokenTier, FeeStructure, OracleGuardRails, OracleSource,
    PositionDirection,
};
use cosmwasm_std::to_binary;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::WasmMsg;
use cosmwasm_std::{coins, Addr, BankMsg, DepsMut, Env, MessageInfo, Response};

pub fn try_initialize_market(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
    market_name: String,
    amm_base_asset_reserve: u128,
    amm_quote_asset_reserve: u128,
    amm_periodicity: i64,
    amm_peg_multiplier: u128,
    oracle_source: OracleSource,
    margin_ratio_initial: u32,
    margin_ratio_partial: u32,
    margin_ratio_maintenance: u32,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let now = env.block.time.seconds();
    let clock_slot = now.clone();

    let existing_market = Markets.load(deps.storage, market_index)?;
    if existing_market.initialized {
        return Err(ContractError::MarketIndexAlreadyInitialized {});
    }
    if amm_base_asset_reserve != amm_quote_asset_reserve {
        return Err(ContractError::InvalidInitialPeg.into());
    }

    let init_mark_price = helpers::amm::calculate_price(
        amm_quote_asset_reserve,
        amm_base_asset_reserve,
        amm_peg_multiplier,
    )?;

    // Verify there's no overflow
    let _k = helpers::bn::U192::from(amm_base_asset_reserve)
        .checked_mul(helpers::bn::U192::from(amm_quote_asset_reserve))
        .ok_or_else(|| return ContractError::MathError {})?;

    // Verify oracle is readable
    //Todo() get the oracle fix here
    // let OraclePriceData {
    //     price: oracle_price,
    //     ..
    // } = match oracle_source {
    //     OracleSource::Oracle => market
    //         .amm
    //         .get_pyth_price(&ctx.accounts.oracle, clock_slot)
    //         .unwrap(),
    //     OracleSource::Switchboard => market
    //         .amm
    //         .get_switchboard_price(&ctx.accounts.oracle, clock_slot)
    //         .unwrap(),
    // };

    // let last_oracle_price_twap = match oracle_source {
    //     OracleSource::Oracle => market.amm.get_pyth_twap(&ctx.accounts.oracle)?,
    //     OracleSource::Simulated => oracle_price,
    // };

    // validate_margin(
    //     margin_ratio_initial,
    //     margin_ratio_initial,
    //     margin_ratio_maintenance,
    // )?;

    // let market = Market {
    //     market_name: market_name,
    //     initialized: true,
    //     base_asset_amount_long: 0,
    //     base_asset_amount_short: 0,
    //     base_asset_amount: 0,
    //     open_interest: 0,
    //     margin_ratio_initial, // unit is 20% (+2 decimal places)
    //     margin_ratio_partial,
    //     margin_ratio_maintenance,
    //     amm: Amm {
    //         oracle: todo!(),
    //         oracle_source,
    //         base_asset_reserve: amm_base_asset_reserve,
    //         quote_asset_reserve: amm_quote_asset_reserve,
    //         cumulative_repeg_rebate_long: 0,
    //         cumulative_repeg_rebate_short: 0,
    //         cumulative_funding_rate_long: 0,
    //         cumulative_funding_rate_short: 0,
    //         last_funding_rate: 0,
    //         last_funding_rate_ts: cast_to_i64(now)?,
    //         funding_period: amm_periodicity,
    //         last_oracle_price_twap: todo!(),
    //         last_mark_price_twap: init_mark_price,
    //         last_mark_price_twap_ts: cast_to_i64(now)?,
    //         sqrt_k: amm_base_asset_reserve,
    //         peg_multiplier: amm_peg_multiplier,
    //         total_fee: 0,
    //         total_fee_minus_distributions: 0,
    //         total_fee_withdrawn: 0,
    //         minimum_quote_asset_trade_size: 10000000,
    //         last_oracle_price_twap_ts: now,
    //         last_oracle_price: oracle_price,
    //         minimum_base_asset_trade_size: 10000000,
    //     },
    // };
    // Markets.save(deps.storage, market_index, &market)?;
    Ok(Response::new().add_attribute("method", "try_initialize_market"))
}

pub fn try_deposit_collateral(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u64,
) -> Result<Response, ContractError> {
    let user_address = info.sender.clone();
    let mut user = Users.load(deps.storage, &user_address)?;
    let now = env.block.time.seconds();

    if amount == 0 {
        return Err(ContractError::InsufficientDeposit.into());
    }

    assert_sent_uusd_balance(&info.clone(), cast_to_u128(amount)?)?;

    let collateral_before = user.collateral;
    let cumulative_deposits_before = user.cumulative_deposits;

    user.collateral = user
        .collateral
        .checked_add(amount as u128)
        .ok_or_else(|| return ContractError::MathError {})?;
    user.cumulative_deposits = user
        .cumulative_deposits
        .checked_add(amount.into())
        .ok_or_else(|| return ContractError::MathError {})?;

    controller::funding::settle_funding_payment(&mut deps, &user_address, cast_to_i64(now)?)?;
    //get and send tokens to collateral vault
    let state = STATE.load(deps.storage)?;
    let bankMsg = BankMsg::Send {
        to_address: state.collateral_vault.into_string(),
        amount: coins(cast_to_u128(amount)?, "uusd"),
    };
    let deposit_history_info_length = DepositHistoryInfo
        .load(deps.storage)?
        .len
        .checked_add(1)
        .ok_or_else(|| (ContractError::MathError))?;
    DepositHistoryInfo.update(
        deps.storage,
        |mut i| -> Result<DepositInfo, ContractError> {
            i.len = deposit_history_info_length;
            Ok(i)
        },
    );
    DepositHistory.save(
        deps.storage,
        (deposit_history_info_length as u64, user_address.clone()),
        &DepositRecord {
            ts: cast_to_i64(now)?,
            record_id: cast(deposit_history_info_length)?,
            user: user_address.clone(),
            direction: DepositDirection::DEPOSIT,
            collateral_before,
            cumulative_deposits_before,
            amount: cast(amount)?,
        },
    )?;
    if state.max_deposit > 0 && user.cumulative_deposits > cast(state.max_deposit)? {
        return Err(ContractError::UserMaxDeposit.into());
    }
    Users.update(
        deps.storage,
        &user_address.clone(),
        |m| -> Result<User, ContractError> { Ok(user) },
    )?;
    Ok(Response::new()
        .add_message(bankMsg)
        .add_attribute("method", "try_deposit_collateral"))
}

pub fn try_withdraw_collateral(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u64,
) -> Result<Response, ContractError> {
    let user_address = info.sender.clone();
    let mut user = Users.load(deps.storage, &user_address)?;
    let now = env.block.time.seconds();

    let collateral_before = user.collateral;
    let cumulative_deposits_before = user.cumulative_deposits;

    controller::funding::settle_funding_payment(&mut deps, &user_address, cast_to_i64(now)?)?;

    if cast_to_u128(amount)? > user.collateral {
        return Err(ContractError::InsufficientCollateral.into());
    }

    let state = STATE.load(deps.storage)?;
    let collateral_balance = query_balance(&deps.querier, state.collateral_vault.clone())?;
    let insurance_balance = query_balance(&deps.querier, state.insurance_vault.clone())?;
    let (collateral_account_withdrawal, insurance_account_withdrawal) =
        calculate_withdrawal_amounts(
            cast(amount)?,
            cast(collateral_balance)?,
            cast(insurance_balance)?,
        )?;

    // amount_withdrawn can be less than amount if there is an insufficient balance in collateral and insurance vault
    let amount_withdraw = collateral_account_withdrawal
        .checked_add(insurance_account_withdrawal)
        .ok_or_else(|| (ContractError::MathError))?;

    user.cumulative_deposits = user
        .cumulative_deposits
        .checked_sub(cast(amount_withdraw)?)
        .ok_or_else(|| (ContractError::MathError))?;

    user.collateral = user
        .collateral
        .checked_sub(cast(collateral_account_withdrawal)?)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_sub(cast(insurance_account_withdrawal)?)
        .ok_or_else(|| (ContractError::MathError))?;

    // TODO: change this to meets initial margin requirement from margin.rs
    // let (_total_collateral, _unrealized_pnl, _base_asset_value, margin_ratio) =
    //     calculate_margin_ratio(&deps, &user_address)?;

    // if margin_ratio < state.margin_ratio_initial {
    //     return Err(ContractError::InsufficientCollateral.into());
    // }

    let mut messages: Vec<CosmosMsg> = vec![];

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.collateral_vault.clone().to_string(),
        msg: to_binary(&VaultInterface::WithdrawFunds {
            to_address: info.sender.to_string(),
            amount: cast(collateral_account_withdrawal)?,
        })?,
        funds: vec![],
    }));

    if insurance_account_withdrawal > 0 {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.insurance_vault.to_string(),
            msg: to_binary(&VaultInterface::WithdrawFunds {
                to_address: info.sender.to_string(),
                amount: cast(insurance_account_withdrawal)?,
            })?,
            funds: vec![],
        }));
    }

    let deposit_history_info_length = DepositHistoryInfo
        .load(deps.storage)?
        .len
        .checked_add(1)
        .ok_or_else(|| (ContractError::MathError))?;
    DepositHistoryInfo.update(
        deps.storage,
        |mut i| -> Result<DepositInfo, ContractError> {
            i.len = deposit_history_info_length;
            Ok(i)
        },
    )?;
    DepositHistory.save(
        deps.storage,
        (deposit_history_info_length as u64, user_address.clone()),
        &DepositRecord {
            ts: cast_to_i64(now)?,
            record_id: cast(deposit_history_info_length)?,
            user: user_address,
            direction: DepositDirection::WITHDRAW,
            collateral_before,
            cumulative_deposits_before,
            amount: cast(amount_withdraw)?,
        },
    )?;
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "try_withdraw_collateral"))
}

pub fn try_open_position(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    direction: PositionDirection,
    quote_asset_amount: u128,
    market_index: u64,
    limit_price: u128,
) -> Result<Response, ContractError> {
    let user_address = info.sender.clone();
    let user = Users.load(deps.storage, &user_address)?;
    let now = env.block.time.seconds();

    controller::funding::settle_funding_payment(&mut deps, &user_address, cast_to_i64(now)?)?;

    let market_position: Position;
    let mut open_position: bool = false;
    let mut position_index: u64 = 0;
    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let mark_position = Positions.load(deps.storage, (&user_address, n))?;
            if is_for(mark_position.clone(), market_index) {
                market_position = mark_position;
                open_position = true;
                //get the position as n and save market data at (addr, n ) index?
                break;
            }
        }
    }

    if open_position {
        let new_market_position = Position {
            market_index,
            base_asset_amount: 0,
            quote_asset_amount: 0,
            last_cumulative_funding_rate: 0,
            last_cumulative_repeg_rebate: 0,
            last_funding_rate_ts: 0,
            stop_profit_price: 0,
            stop_profit_amount: 0,
            stop_loss_price: 0,
            stop_loss_amount: 0,
            transfer_to: Addr::unchecked("".to_string()),
        };
    }

    let market_position = Positions.load(deps.storage, (&user_address, position_index))?;

    let mut potentially_risk_increasing = true;

    let mark_price_before: u128;
    let oracle_mark_spread_pct_before: i128;
    let is_oracle_valid: bool;

    let market = Markets.load(deps.storage, market_index)?;
    let mark_price_before = helpers::amm::calculate_price(
        market.amm.quote_asset_reserve,
        market.amm.base_asset_reserve,
        market.amm.peg_multiplier,
    )?;

    Ok(Response::new().add_attribute("method", "try_open_position"))
}

pub fn try_close_position(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
    let user_address = info.sender.clone();
    let user = Users.load(deps.storage, &user_address)?;
    let now = env.block.time.seconds();
    let clock_slot = now.clone();
    controller::funding::settle_funding_payment(&mut deps, &user_address, cast_to_i64(now)?)?;

    let mut market_position: Position;
    let mut open_position: bool = false;
    let position_index: u64;
    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let mark_position = Positions.load(deps.storage, (&user_address, n))?;
            if is_for(mark_position.clone(), market_index) {
                market_position = mark_position;
                open_position = true;
                //get the position as n and save market data at (addr, n ) index?
                break;
            }
        }
    }

    if open_position {
        return Err(ContractError::UserHasNoPositionInMarket.into());
    }
    let market = Markets.load(deps.storage, market_index)?;
    let mark_price_before = helpers::amm::calculate_price(
        market.amm.quote_asset_reserve,
        market.amm.base_asset_reserve,
        market.amm.peg_multiplier,
    )?;

    //price oracle address
    let price_oracle = Addr::unchecked("".to_string());
    let (oracle_price, _, oracle_mark_spread_pct_before) =
        helpers::amm::calculate_oracle_mark_spread_pct(
            &market.amm,
            &price_oracle,
            0,
            clock_slot,
            Some(mark_price_before),
        )?;

    // let direction_to_close =
    //     helpers::position::direction_to_close_position(market_position.base_asset_amount);
    // let (quote_asset_amount, base_asset_amount) = controller::position::close(
    //     &mut deps,
    //     &user_address,
    //     market_index,
    //     position_index,
    //     cast_to_i64(now)?,
    // )?;
    // let base_asset_amount = base_asset_amount.unsigned_abs();
    //optional account TODO
    Ok(Response::new().add_attribute("method", "try_close_position"))
}

//new limit order interfaces
pub fn try_place_order(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order: Order,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_place_order"))
}

pub fn try_cancel_order(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_id: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_cancel_order"))
}

pub fn try_cancel_order_by_user_id(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_order_id: u8,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_cancel_order_by_user_id"))
}
pub fn try_expire_orders(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_expire_orders"))
}

pub fn try_fill_order(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_id: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_fill_order"))
}

pub fn try_place_and_fill_order(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order: Order,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_place_and_fill_order"))
}

pub fn try_liquidate(
    deps: DepsMut,
    info: MessageInfo,
    user: String,
    market_index: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_liquidate"))
}

pub fn try_move_amm_price(
    mut deps: DepsMut,
    info: MessageInfo,
    base_asset_reserve: u128,
    quote_asset_reserve: u128,
    market_index: u64,
) -> Result<Response, ContractError> {
    controller::amm::move_price(
        &mut deps,
        market_index,
        base_asset_reserve,
        quote_asset_reserve,
    )?;
    Ok(Response::new().add_attribute("method", "try_move_amm_price"))
}

pub fn try_withdraw_fees(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    amount: u64,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let state = STATE.load(deps.storage)?;
    let mut market = Markets.load(deps.storage, market_index)?;

    // A portion of fees must always remain in protocol to be used to keep markets optimal
    let max_withdraw = market
        .amm
        .total_fee
        .checked_mul(SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_NUMERATOR)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_div(SHARE_OF_FEES_ALLOCATED_TO_CLEARING_HOUSE_DENOMINATOR)
        .ok_or_else(|| (ContractError::MathError))?
        .checked_sub(market.amm.total_fee_withdrawn)
        .ok_or_else(|| (ContractError::MathError))?;

    if cast_to_u128(amount)? > max_withdraw {
        return Err(ContractError::AdminWithdrawTooLarge.into());
    }

    let message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.collateral_vault.to_string(),
        msg: to_binary(&VaultInterface::WithdrawFunds {
            to_address: info.sender.to_string(),
            amount: cast(amount)?,
        })?,
        funds: vec![],
    });

    market.amm.total_fee_withdrawn = market
        .amm
        .total_fee_withdrawn
        .checked_add(cast(amount)?)
        .ok_or_else(|| (ContractError::MathError))?;

    Ok(Response::new().add_attribute("method", "try_withdraw_fees"))
}

pub fn try_withdraw_from_insurance_vault_to_market(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    amount: u64,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let state = STATE.load(deps.storage)?;

    let mut market = Markets.load(deps.storage, market_index)?;
    market.amm.total_fee_minus_distributions = market
        .amm
        .total_fee_minus_distributions
        .checked_add(cast(amount)?)
        .ok_or_else(|| (ContractError::MathError))?;
    Markets.update(
        deps.storage,
        market_index,
        |m| -> Result<Market, ContractError> { Ok(market) },
    )?;

    let message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.insurance_vault.to_string(),
        msg: to_binary(&VaultInterface::WithdrawFunds {
            to_address: state.collateral_vault.to_string(),
            amount: cast(amount)?,
        })?,
        funds: vec![],
    });
    Ok(Response::new()
        .add_message(message)
        .add_attribute("method", "try_withdraw_from_insurance_vault_to_market"))
}

pub fn try_repeg_amm_curve(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_peg_candidate: u128,
    market_index: u64,
) -> Result<Response, ContractError> {
    let now = env.block.time.seconds();
    let clock_slot = now.clone();
    let market = Markets.load(deps.storage, market_index)?;

    let peg_multiplier_before = market.amm.peg_multiplier;
    let base_asset_reserve_before = market.amm.base_asset_reserve;
    let quote_asset_reserve_before = market.amm.quote_asset_reserve;
    let sqrt_k_before = market.amm.sqrt_k;

    //TODO:: set price_oracle
    let price_oracle = Addr::unchecked("".to_string());

    let adjustment_cost = controller::repeg::repeg(
        &mut deps,
        market_index,
        &price_oracle,
        new_peg_candidate,
        clock_slot,
    )
    .unwrap();
    let peg_multiplier_after = market.amm.peg_multiplier;
    let base_asset_reserve_after = market.amm.base_asset_reserve;
    let quote_asset_reserve_after = market.amm.quote_asset_reserve;
    let sqrt_k_after = market.amm.sqrt_k;

    let curve_history_info_length = CurveHistoryInfo
        .load(deps.storage)?
        .len
        .checked_add(1)
        .ok_or_else(|| (ContractError::MathError))?;
    CurveHistoryInfo.update(
        deps.storage,
        |mut i: CurveInfo| -> Result<CurveInfo, ContractError> {
            i.len = curve_history_info_length;
            Ok(i)
        },
    )?;

    //TODO:: oracle price fetch and trade record impl
    let oracle_price = 100;
    let trade_record = 100;
    CurveHistory.save(
        deps.storage,
        curve_history_info_length as u64,
        &CurveRecord {
            ts: cast_to_i64(now)?,
            record_id: curve_history_info_length as u128,
            market_index,
            peg_multiplier_before,
            base_asset_reserve_before,
            quote_asset_reserve_before,
            sqrt_k_before,
            peg_multiplier_after,
            base_asset_reserve_after,
            quote_asset_reserve_after,
            sqrt_k_after,
            base_asset_amount_long: market.base_asset_amount_long.unsigned_abs(),
            base_asset_amount_short: market.base_asset_amount_short.unsigned_abs(),
            base_asset_amount: market.base_asset_amount,
            open_interest: market.open_interest,
            total_fee: market.amm.total_fee,
            total_fee_minus_distributions: market.amm.total_fee_minus_distributions,
            adjustment_cost,
            oracle_price,
            trade_record,
        },
    )?;
    Ok(Response::new().add_attribute("method", "try_repeg_amm_curve"))
}

pub fn try_update_amm_oracle_twap(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_amm_oracle_twap"))
}

pub fn try_reset_amm_oracle_twap(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_reset_amm_oracle_twap"))
}

pub fn try_settle_funding_payment(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let now = env.block.time.seconds();
    let user_address = info.sender;

    controller::funding::settle_funding_payment(&mut deps, &user_address, cast_to_i64(now)?)?;
    Ok(Response::new().add_attribute("method", "try_settle_funding_payment"))
}
pub fn try_update_funding_rate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {

    let market = Markets.load(deps.storage, market_index)?;
    let now = env.block.time.seconds();
    let clock_slot = now.clone();
    //TODO:: add oracle address
    let price_oracle = Addr::unchecked("".to_string());
    let funding_paused = STATE.load(deps.storage).unwrap().funding_paused;
    controller::funding::update_funding_rate(
        &mut deps,
        market_index,
        &price_oracle,
        cast_to_i64(now)?,
        clock_slot,
        funding_paused,
    )?;
    Ok(Response::new().add_attribute("method", "try_update_funding_rate"))
}

pub fn try_update_k(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
    sqrt_k: u128,
) -> Result<Response, ContractError> {
    let now = env.block.time.seconds();
    let mut market = Markets.load(deps.storage, market_index)?;

    let base_asset_amount_long = market.base_asset_amount_long.unsigned_abs();
    let base_asset_amount_short = market.base_asset_amount_short.unsigned_abs();
    let base_asset_amount = market.base_asset_amount;
    let open_interest = market.open_interest;

    let price_before = helpers::amm::calculate_price(
        market.amm.quote_asset_reserve,
        market.amm.base_asset_reserve,
        market.amm.peg_multiplier,
    )?;

    let peg_multiplier_before = market.amm.peg_multiplier;
    let base_asset_reserve_before = market.amm.base_asset_reserve;
    let quote_asset_reserve_before = market.amm.quote_asset_reserve;
    let sqrt_k_before = market.amm.sqrt_k;

    let adjustment_cost =
        controller::amm::adjust_k_cost(&mut deps, market_index, helpers::bn::U256::from(sqrt_k))?;

    if adjustment_cost > 0 {
        let max_cost = market
            .amm
            .total_fee_minus_distributions
            .checked_sub(market.amm.total_fee_withdrawn)
            .ok_or_else(|| ContractError::MathError {})?;
        if adjustment_cost.unsigned_abs() > max_cost {
            return Err(ContractError::InvalidUpdateK.into());
        } else {
            market.amm.total_fee_minus_distributions = market
                .amm
                .total_fee_minus_distributions
                .checked_sub(adjustment_cost.unsigned_abs())
                .ok_or_else(|| ContractError::MathError {})?;
        }
    } else {
        market.amm.total_fee_minus_distributions = market
            .amm
            .total_fee_minus_distributions
            .checked_add(adjustment_cost.unsigned_abs())
            .ok_or_else(|| ContractError::MathError {})?;
    }

    let amm = &market.amm;
    let price_after = helpers::amm::calculate_price(
        amm.quote_asset_reserve,
        amm.base_asset_reserve,
        amm.peg_multiplier,
    )?;

    let price_change_too_large = cast_to_i128(price_before)?
        .checked_sub(cast_to_i128(price_after)?)
        .ok_or_else(|| ContractError::MathError {})?
        .unsigned_abs()
        .gt(&UPDATE_K_ALLOWED_PRICE_CHANGE);

    if price_change_too_large {
        return Err(ContractError::InvalidUpdateK.into());
    }

    let peg_multiplier_after = amm.peg_multiplier;
    let base_asset_reserve_after = amm.base_asset_reserve;
    let quote_asset_reserve_after = amm.quote_asset_reserve;
    let sqrt_k_after = amm.sqrt_k;

    let total_fee = amm.total_fee;
    let total_fee_minus_distributions = amm.total_fee_minus_distributions;
    let curve_history_info_length = CurveHistoryInfo
        .load(deps.storage)?
        .len
        .checked_add(1)
        .ok_or_else(|| (ContractError::MathError))?;
    CurveHistoryInfo.update(
        deps.storage,
        |mut i: CurveInfo| -> Result<CurveInfo, ContractError> {
            i.len = curve_history_info_length;
            Ok(i)
        },
    );
    //TODO:: oracle price fetch and trade record impl
    let oracle_price = 100;
    let trade_record = 100;
    CurveHistory.save(
        deps.storage,
        curve_history_info_length as u64,
        &CurveRecord {
            ts: cast_to_i64(now)?,
            record_id: curve_history_info_length as u128,
            market_index,
            peg_multiplier_before,
            base_asset_reserve_before,
            quote_asset_reserve_before,
            sqrt_k_before,
            peg_multiplier_after,
            base_asset_reserve_after,
            quote_asset_reserve_after,
            sqrt_k_after,
            base_asset_amount_long,
            base_asset_amount_short,
            base_asset_amount,
            open_interest,
            adjustment_cost,
            total_fee,
            total_fee_minus_distributions,
            oracle_price,
            trade_record,
        },
    );
    Ok(Response::new().add_attribute("method", "try_update_k"))
}

pub fn try_update_market_minimum_trade_size(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    minimum_trade_size: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let mut market = Markets.load(deps.storage, market_index)?;
    // market.amm.minimum_trade_size = minimum_trade_size;
    Markets.update(
        deps.storage,
        market_index,
        |m| -> Result<Market, ContractError> { Ok(market) },
    )?;
    Ok(Response::new().add_attribute("method", "try_update_market_minimum_trade_size"))
}

pub fn try_update_margin_ratio(
    deps: DepsMut,
    info: MessageInfo,
    margin_ratio_initial: u128,
    margin_ratio_partial: u128,
    margin_ratio_maintenance: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.margin_ratio_initial = margin_ratio_initial;
        state.margin_ratio_partial = margin_ratio_partial;
        state.margin_ratio_maintenance = margin_ratio_maintenance;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_update_margin_ratio"))
}

pub fn try_update_partial_liquidation_close_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.partial_liquidation_close_percentage_numerator = numerator;
        state.partial_liquidation_close_percentage_denominator = denominator;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_update_partial_liquidation_close_percentage"))
}

pub fn try_update_partial_liquidation_penalty_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.partial_liquidation_penalty_percentage_numerator = numerator;
        state.partial_liquidation_penalty_percentage_denominator = denominator;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute(
        "method",
        "try_update_partial_liquidation_penalty_percentage",
    ))
}

pub fn try_update_full_liquidation_penalty_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.full_liquidation_penalty_percentage_numerator = numerator;
        state.full_liquidation_penalty_percentage_denominator = denominator;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_update_full_liquidation_penalty_percentage"))
}

pub fn try_update_partial_liquidation_liquidator_share_denominator(
    deps: DepsMut,
    info: MessageInfo,
    denominator: u64,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.partial_liquidation_liquidator_share_denominator = denominator;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute(
        "method",
        "try_update_partial_liquidation_liquidator_share_denominator",
    ))
}

pub fn try_update_full_liquidation_liquidator_share_denominator(
    deps: DepsMut,
    info: MessageInfo,
    denominator: u64,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.full_liquidation_liquidator_share_denominator = denominator;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute(
        "method",
        "try_update_full_liquidation_liquidator_share_denominator",
    ))
}

pub fn try_update_fee(
    deps: DepsMut,
    info: MessageInfo,
    fee_numerator: u128,
    fee_denominator: u128,

    first_tier: DiscountTokenTier,
    second_tier: DiscountTokenTier,
    third_tier: DiscountTokenTier,
    fourth_tier: DiscountTokenTier,

    referrer_reward_numerator: u128,
    referrer_reward_denominator: u128,
    referee_discount_numerator: u128,
    referee_discount_denominator: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let fee_structure = FeeStructure {
        fee_numerator,
        fee_denominator,
        first_tier,
        second_tier,
        third_tier,
        fourth_tier,
        referrer_reward_numerator,
        referrer_reward_denominator,
        referee_discount_numerator,
        referee_discount_denominator,
    };
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.fee_structure = fee_structure;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_update_fee"))
}

pub fn try_update_order_filler_reward_structure(
    deps: DepsMut,
    info: MessageInfo,
    reward_numerator: u128,
    reward_denominator: u128,
    time_based_reward_lower_bound: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_order_filler_reward_structure"))
}
pub fn try_update_market_oracle(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    oracle: String,
    oracle_source: OracleSource,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_market_oracle"))
}

pub fn try_update_oracle_guard_rails(
    deps: DepsMut,
    info: MessageInfo,
    use_for_liquidations: bool,
    mark_oracle_divergence_numerator: u128,
    mark_oracle_divergence_denominator: u128,
    slots_before_stale: i64,
    confidence_interval_max_size: u128,
    too_volatile_ratio: i128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let oracle_gr = OracleGuardRails {
        use_for_liquidations,
        mark_oracle_divergence_numerator,
        mark_oracle_divergence_denominator,
        slots_before_stale,
        confidence_interval_max_size,
        too_volatile_ratio,
    };
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.oracle_guard_rails = oracle_gr;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_update_oracle_guard_rails"))
}

pub fn try_update_max_deposit(
    deps: DepsMut,
    info: MessageInfo,
    max_deposit: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.max_deposit = max_deposit;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_max_deposit"))
}

pub fn try_update_exchange_paused(
    deps: DepsMut,
    info: MessageInfo,
    exchange_paused: bool,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.exchange_paused = exchange_paused;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_exchange_paused"))
}

pub fn try_disable_admin_control_prices(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.admin_controls_prices = false;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_disable_admin_control_prices"))
}
pub fn try_update_funding_paused(
    deps: DepsMut,
    info: MessageInfo,
    funding_paused: bool,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.funding_paused = funding_paused;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_update_funding_paused"))
}

pub fn try_update_market_minimum_quote_asset_trade_size(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    minimum_trade_size: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let mut market = Markets.load(deps.storage, market_index)?;
    // market.amm.minimum_trade_size = minimum_trade_size;
    Markets.update(
        deps.storage,
        market_index,
        |m| -> Result<Market, ContractError> { Ok(market) },
    )?;
    Ok(Response::new().add_attribute("method", "try_update_market_minimum_quote_asset_trade_size"))
}

pub fn try_update_market_minimum_base_asset_trade_size(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    minimum_trade_size: u128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender.clone())?;
    let mut market = Markets.load(deps.storage, market_index)?;
    // market.amm.minimum_trade_size = minimum_trade_size;
    Markets.update(
        deps.storage,
        market_index,
        |m| -> Result<Market, ContractError> { Ok(market) },
    )?;
    Ok(Response::new().add_attribute("method", "try_update_market_minimum_base_asset_trade_size"))
}
