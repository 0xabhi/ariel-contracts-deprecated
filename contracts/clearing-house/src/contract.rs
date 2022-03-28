use cosmwasm_std::{
    coins, entry_point, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult,
};
use cosmwasm_std::{CosmosMsg, WasmMsg};
use cw2::set_contract_version;

use crate::controller;
use crate::controller::margin::calculate_margin_ratio;
use crate::helpers;
// use crate::helpers::casting::cast_to_i64;
use crate::helpers::casting::*;
use crate::helpers::constants::*;
use crate::helpers::withdrawal::calculate_withdrawal_amounts;
use crate::states::curve_history::*;
use crate::states::liquidation_history::{LiquidationHistory, LiquidationHistoryInfo};
use crate::states::market::{Market, Markets};
use crate::states::state::{State, STATE};
use crate::states::trade_history::{TradeHistory, TradeHistoryInfo};
use crate::states::user::{is_for, Position, Positions, User, Users};
use crate::states::{deposit_history::*, funding_history::*};

use ariel::execute::{ExecuteMsg, InstantiateMsg};
use ariel::helper::{
    addr_validate_to_lower, assert_sent_uusd_balance, query_balance, VaultInterface,
};
use ariel::queries::QueryMsg;
use ariel::response::*;
use ariel::types::{
    DepositDirection, DiscountTokenTier, FeeStructure, OracleGuardRails, PositionDirection,
};

use crate::error::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:clearing-house";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    //TODO:: adding condition to check the initialization, if it's done already
    let fs = FeeStructure {
        fee_numerator: 0,
        fee_denominator: 0,
        first_tier: DiscountTokenTier {
            minimum_balance: 0,
            discount_numerator: 0,
            discount_denominator: 0,
        },

        second_tier: DiscountTokenTier {
            minimum_balance: 0,
            discount_numerator: 0,
            discount_denominator: 0,
        },
        third_tier: DiscountTokenTier {
            minimum_balance: 0,
            discount_numerator: 0,
            discount_denominator: 0,
        },
        fourth_tier: DiscountTokenTier {
            minimum_balance: 0,
            discount_numerator: 0,
            discount_denominator: 0,
        },
        referrer_reward_numerator: 0,
        referrer_reward_denominator: 0,
        referee_discount_numerator: 0,
        referee_discount_denominator: 0,
    };
    let oracle_gr = OracleGuardRails {
        use_for_liquidations: true,
        mark_oracle_divergence_numerator: 0,
        mark_oracle_divergence_denominator: 0,
        slots_before_stale: 0,
        confidence_interval_max_size: 0,
        too_volatile_ratio: 0,
    };
    let state = State {
        admin: addr_validate_to_lower(deps.api, &msg.admin).unwrap(),
        exchange_paused: true,
        funding_paused: true,
        admin_controls_prices: true,
        collateral_vault: addr_validate_to_lower(deps.api, &msg.collateral_vault).unwrap(),
        insurance_vault: addr_validate_to_lower(deps.api, &msg.insurance_vault).unwrap(),
        margin_ratio_initial: 0,
        margin_ratio_maintenance: 0,
        margin_ratio_partial: 0,
        partial_liquidation_close_percentage_numerator: 0,
        partial_liquidation_close_percentage_denominator: 0,
        partial_liquidation_penalty_percentage_numerator: 0,
        partial_liquidation_penalty_percentage_denominator: 0,
        full_liquidation_penalty_percentage_numerator: 0,
        full_liquidation_penalty_percentage_denominator: 0,
        partial_liquidation_liquidator_share_denominator: 0,
        full_liquidation_liquidator_share_denominator: 0,
        max_deposit: 1000000,
        fee_structure: fs,
        oracle_guard_rails: oracle_gr,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitializeMarket {
            market_index,
            market_name,
            amm_base_asset_reserve,
            amm_quote_asset_reserve,
            amm_periodicity,
            amm_peg_multiplier,
        } => try_initialize_market(
            deps,
            _env,
            info,
            market_index,
            market_name,
            amm_base_asset_reserve,
            amm_quote_asset_reserve,
            amm_periodicity,
            amm_peg_multiplier,
        ),
        ExecuteMsg::DepositCollateral { amount } => {
            try_deposit_collateral(deps, _env, info, amount)
        }
        ExecuteMsg::WithdrawCollateral { amount } => {
            try_withdraw_collateral(deps, _env, info, amount)
        }
        ExecuteMsg::OpenPosition {
            direction,
            quote_asset_amount,
            market_index,
            limit_price,
        } => try_open_position(
            deps,
            _env,
            info,
            direction,
            quote_asset_amount,
            market_index,
            limit_price,
        ),
        ExecuteMsg::ClosePosition { market_index } => {
            try_close_position(deps, _env, info, market_index)
        }
        ExecuteMsg::Liquidate { user, market_index } => {
            try_liquidate(deps, info, user, market_index)
        }
        ExecuteMsg::MoveAMMPrice {
            base_asset_reserve,
            quote_asset_reserve,
            market_index,
        } => try_move_amm_price(
            deps,
            info,
            base_asset_reserve,
            quote_asset_reserve,
            market_index,
        ),
        ExecuteMsg::WithdrawFees {
            market_index,
            amount,
        } => try_withdraw_fees(deps, info, market_index, amount),
        ExecuteMsg::WithdrawFromInsuranceVaultToMarket {
            market_index,
            amount,
        } => try_withdraw_from_insurance_vault_to_market(deps, info, market_index, amount),
        ExecuteMsg::RepegAMMCurve {
            new_peg_candidate,
            market_index,
        } => try_repeg_amm_curve(deps, _env, info, new_peg_candidate, market_index),
        ExecuteMsg::SettleFundingPayment {} => try_settle_funding_payment(deps, _env, info),
        ExecuteMsg::UpdateFundingRate { market_index } => {
            try_update_funding_rate(deps, _env, info, market_index)
        }
        ExecuteMsg::UpdateK {
            market_index,
            sqrt_k,
        } => try_update_k(deps, _env, info, market_index, sqrt_k),
        ExecuteMsg::UpdateMarketMinimumTradeSize {
            market_index,
            minimum_trade_size,
        } => try_update_market_minimum_trade_size(deps, info, market_index, minimum_trade_size),
        ExecuteMsg::UpdateMarginRatio {
            margin_ratio_initial,
            margin_ratio_partial,
            margin_ratio_maintenance,
        } => try_update_margin_ratio(
            deps,
            info,
            margin_ratio_initial,
            margin_ratio_partial,
            margin_ratio_maintenance,
        ),
        ExecuteMsg::UpdatePartialLiquidationClosePercentage {
            numerator,
            denominator,
        } => try_update_partial_liquidation_close_percentage(deps, info, numerator, denominator),
        ExecuteMsg::UpdatePartialLiquidationPenaltyPercentage {
            numerator,
            denominator,
        } => try_update_partial_liquidation_penalty_percentage(deps, info, numerator, denominator),
        ExecuteMsg::UpdateFullLiquidationPenaltyPercentage {
            numerator,
            denominator,
        } => try_update_full_liquidation_penalty_percentage(deps, info, numerator, denominator),
        ExecuteMsg::UpdatePartialLiquidationLiquidatorShareDenominator { denominator } => {
            try_update_partial_liquidation_liquidator_share_denominator(deps, info, denominator)
        }
        ExecuteMsg::UpdateFullLiquidationLiquidatorShareDenominator { denominator } => {
            try_update_full_liquidation_liquidator_share_denominator(deps, info, denominator)
        }
        ExecuteMsg::UpdateFee {
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
        } => try_update_fee(
            deps,
            info,
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
        ),
        ExecuteMsg::UpdateOraceGuardRails {
            use_for_liquidations,
            mark_oracle_divergence_numerator,
            mark_oracle_divergence_denominator,
            slots_before_stale,
            confidence_interval_max_size,
            too_volatile_ratio,
        } => try_update_oracle_guard_rails(
            deps,
            info,
            use_for_liquidations,
            mark_oracle_divergence_numerator,
            mark_oracle_divergence_denominator,
            slots_before_stale,
            confidence_interval_max_size,
            too_volatile_ratio,
        ),
        ExecuteMsg::UpdateAdmin { admin } => try_update_admin(deps, info, admin),
        ExecuteMsg::UpdateMaxDeposit { max_deposit } => {
            try_update_max_deposit(deps, info, max_deposit)
        }
        ExecuteMsg::UpdateExchangePaused { exchange_paused } => {
            try_update_exchange_paused(deps, info, exchange_paused)
        }
        ExecuteMsg::DisableAdminControlsPrices {} => try_disable_admin_control_prices(deps, info),
        ExecuteMsg::UpdateFundingPaused { funding_paused } => {
            try_update_funding_paused(deps, info, funding_paused)
        }
    }
}

pub fn try_initialize_market(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
    market_name: String,
    amm_base_asset_reserve: u128,
    amm_quote_asset_reserve: u128,
    amm_periodicity: u128,
    amm_peg_multiplier: u128,
) -> Result<Response, ContractError> {
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

    //TODO:: oracle price check
    let (_, oracle_price_twap, _, _, _) = market
        .amm
        .get_oracle_price(&ctx.accounts.oracle, clock_slot)
        .unwrap();

    let market = Market {
        market_name: market_name,
        initialized: true,
        base_asset_amount_long: 0,
        base_asset_amount_short: 0,
        base_asset_amount: 0,
        open_interest: 0,
        amm: AMM {
            oracle: *ctx.accounts.oracle.key, //TODO add oracle address
            oracle_source: OracleSource::Pyth,
            base_asset_reserve: amm_base_asset_reserve,
            quote_asset_reserve: amm_quote_asset_reserve,
            cumulative_repeg_rebate_long: 0,
            cumulative_repeg_rebate_short: 0,
            cumulative_funding_rate_long: 0,
            cumulative_funding_rate_short: 0,
            last_funding_rate: 0,
            last_funding_rate_ts: now,
            funding_period: amm_periodicity,
            last_oracle_price_twap: oracle_price_twap,
            last_mark_price_twap: init_mark_price,
            last_mark_price_twap_ts: now,
            sqrt_k: amm_base_asset_reserve,
            peg_multiplier: amm_peg_multiplier,
            total_fee: 0,
            total_fee_withdrawn: 0,
            total_fee_minus_distributions: 0,
            minimum_trade_size: 10000000,
            last_oracle_price_twap_ts: now,
        },
    };
    Markets.save(deps.storage, market_index, &market)?;
    Ok(Response::new().add_attribute("method", "try_initialize_market"))
}

pub fn try_deposit_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: i128,
) -> Result<Response, ContractError> {
    let user_address = info.sender.clone();
    let user = Users.load(deps.storage, &user_address)?;
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
        .checked_add(amount)
        .ok_or_else(|| return ContractError::MathError {})?;

    controller::funding::settle_funding_payment(deps, &user_address, cast_to_i64(now)?)?;
    //get and send tokens to collateral v
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
    DepositHistoryInfo.update(deps.storage, |i| -> Result<DepositInfo, ContractError> {
        i.len = deposit_history_info_length;
        Ok(i)
    });
    DepositHistory.save(
        deps.storage,
        (deposit_history_info_length as u64, user_address),
        &DepositRecord {
            ts: cast_to_i64(now)?,
            record_id: cast(deposit_history_info_length)?,
            user: user_address,
            direction: DepositDirection::DEPOSIT,
            collateral_before,
            cumulative_deposits_before,
            amount: cast(amount)?,
        },
    )?;
    if state.max_deposit > 0 && user.cumulative_deposits > cast(state.max_deposit)? {
        return Err(ContractError::UserMaxDeposit.into());
    }
    Ok(Response::new()
        .add_message(bankMsg)
        .add_attribute("method", "try_deposit_collateral"))
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: i128,
) -> Result<Response, ContractError> {
    let user_address = info.sender.clone();
    let user = Users.load(deps.storage, &user_address)?;
    let now = env.block.time.seconds();

    let collateral_before = user.collateral;
    let cumulative_deposits_before = user.cumulative_deposits;

    controller::funding::settle_funding_payment(deps, &user_address, cast_to_i64(now)?)?;

    if cast_to_u128(amount)? > user.collateral {
        return Err(ContractError::InsufficientCollateral.into());
    }

    let state = STATE.load(deps.storage)?;
    let collateral_balance = query_balance(&deps.querier, state.collateral_vault)?;
    let insurance_balance = query_balance(&deps.querier, state.insurance_vault)?;
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

    let (_total_collateral, _unrealized_pnl, _base_asset_value, margin_ratio) =
        calculate_margin_ratio(deps, &user_address)?;

    if margin_ratio < state.margin_ratio_initial {
        return Err(ContractError::InsufficientCollateral.into());
    }
    let mut messages: Vec<CosmosMsg> = vec![];

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.collateral_vault.to_string(),
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
    DepositHistoryInfo.update(deps.storage, |i| -> Result<DepositInfo, ContractError> {
        i.len = deposit_history_info_length;
        Ok(i)
    });
    DepositHistory.save(
        deps.storage,
        (deposit_history_info_length as u64, user_address),
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
    deps: DepsMut,
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

    controller::funding::settle_funding_payment(deps, &user_address, cast_to_i64(now)?)?;

    let market_position: Position;
    let open_position: bool = false;
    let position_index: u64;
    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let mark_position = Positions.load(deps.storage, (&user_address, n))?;
            if is_for(mark_position, market_index) {
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
    let user_address = info.sender.clone();
    let user = Users.load(deps.storage, &user_address)?;
    let now = env.block.time.seconds();
    let clock_slot = now.clone();
    controller::funding::settle_funding_payment(deps, &user_address, cast_to_i64(now)?)?;

    let market_position: Position;
    let open_position: bool = false;
    let position_index: u64;
    if user.positions_length > 0 {
        for n in 1..user.positions_length {
            let mark_position = Positions.load(deps.storage, (&user_address, n))?;
            if is_for(mark_position, market_index) {
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

    let direction_to_close =
        helpers::position::direction_to_close_position(market_position.base_asset_amount);
    let (quote_asset_amount, base_asset_amount) = controller::position::close(
        deps,
        &user_address,
        market_index,
        position_index,
        cast_to_i64(now)?,
    )?;
    let base_asset_amount = base_asset_amount.unsigned_abs();
    //optional account TODO
    Ok(Response::new().add_attribute("method", "try_close_position"))
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
    deps: DepsMut,
    info: MessageInfo,
    base_asset_reserve: u128,
    quote_asset_reserve: u128,
    market_index: u64,
) -> Result<Response, ContractError> {
    controller::amm::move_price(deps, market_index, base_asset_reserve, quote_asset_reserve)?;
    Ok(Response::new().add_attribute("method", "try_move_amm_price"))
}

pub fn try_withdraw_fees(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    amount: u64,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let market = Markets.load(deps.storage, market_index)?;

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
    let state = STATE.load(deps.storage)?;
    if info.sender != state.admin {
        return Err(ContractError::Unauthorized {});
    }

    let market = Markets.load(deps.storage, market_index)?;
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
    deps: DepsMut,
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
        deps,
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
        |i: CurveInfo| -> Result<CurveInfo, ContractError> {
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
    );
    Ok(Response::new().add_attribute("method", "try_repeg_amm_curve"))
}

pub fn try_settle_funding_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let now = env.block.time.seconds();
    let user_address = info.sender;

    controller::funding::settle_funding_payment(deps, &user_address, cast_to_i64(now)?)?;
    Ok(Response::new().add_attribute("method", "try_settle_funding_payment"))
}
pub fn try_update_funding_rate(
    deps: DepsMut,
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
        deps,
        market_index,
        &price_oracle,
        cast_to_i64(now)?,
        clock_slot,
        funding_paused,
    )?;
    Ok(Response::new().add_attribute("method", "try_update_funding_rate"))
}

pub fn try_update_k(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_index: u64,
    sqrt_k: u128,
) -> Result<Response, ContractError> {
    let now = env.block.time.seconds();
    let market = Markets.load(deps.storage, market_index)?;

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
        controller::amm::adjust_k_cost(deps, market_index, helpers::bn::U256::from(sqrt_k))?;

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
        |i: CurveInfo| -> Result<CurveInfo, ContractError> {
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
    let market = Markets.load(deps.storage, market_index)?;
    market.amm.minimum_trade_size = minimum_trade_size;
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
        state.fee_structure = fee_structure;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_update_fee"))
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
    let oracle_gr = OracleGuardRails {
        use_for_liquidations,
        mark_oracle_divergence_numerator,
        mark_oracle_divergence_denominator,
        slots_before_stale,
        confidence_interval_max_size,
        too_volatile_ratio,
    };
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
        state.oracle_guard_rails = oracle_gr;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_update_oracle_guard_rails"))
}
pub fn try_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let admin_addr = addr_validate_to_lower(deps.api, &admin)?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
        state.admin = admin_addr;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_update_admin"))
}

pub fn try_update_max_deposit(
    deps: DepsMut,
    info: MessageInfo,
    max_deposit: u128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
        state.exchange_paused = exchange_paused;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_exchange_paused"))
}

pub fn try_disable_admin_control_prices(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
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
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }
        state.funding_paused = funding_paused;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_update_funding_paused"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUser { user_address } => to_binary(&get_user(deps, user_address)?),
        QueryMsg::GetUserMarketPosition {
            user_address,
            index,
        } => to_binary(&get_user_position(deps, user_address, index)?),
        QueryMsg::GetAdmin {} => to_binary(&get_admin(deps)?),
        QueryMsg::IsExchangePaused {} => to_binary(&is_exchange_paused(deps)?),
        QueryMsg::IsFundingPaused {} => to_binary(&is_funding_paused(deps)?),
        QueryMsg::AdminControlsPrices {} => to_binary(&admin_controls_prices(deps)?),
        QueryMsg::GetVaults {} => to_binary(&get_vaults_address(deps)?),
        QueryMsg::GetMarginRatio {} => to_binary(&get_margin_ratios(deps)?),
        QueryMsg::GetPartialLiquidationClosePercentage {} => {
            to_binary(&get_partial_liquidation_close_percentage(deps)?)
        }
        QueryMsg::GetPartialLiquidationPenaltyPercentage {} => {
            to_binary(&get_partial_liquidation_penalty_percentage(deps)?)
        }
        QueryMsg::GetFullLiquidationPenaltyPercentage {} => {
            to_binary(&get_full_liquidation_penalty_percentage(deps)?)
        }
        QueryMsg::GetPartialLiquidatorSharePercentage {} => {
            to_binary(&get_partial_liquidator_share_percentage(deps)?)
        }
        QueryMsg::GetFullLiquidatorSharePercentage {} => {
            to_binary(&get_full_liquidator_share_percentage(deps)?)
        }
        QueryMsg::GetMaxDepositLimit {} => to_binary(&get_max_deposit_limit(deps)?),
        QueryMsg::GetFeeStructure {} => to_binary(&get_fee_structure(deps)?),
        QueryMsg::GetCurveHistoryLength {} => to_binary(&get_curve_history_length(deps)?),
        QueryMsg::GetCurveHistory { index } => to_binary(&get_curve_history(deps, index)?),
        QueryMsg::GetDepositHistoryLength {} => to_binary(&get_deposit_history_length(deps)?),
        QueryMsg::GetDepositHistory {
            user_address,
            index,
        } => to_binary(&get_deposit_history(deps, user_address, index)?),
        QueryMsg::GetFundingPaymentHistoryLength {} => {
            to_binary(&get_funding_payment_history_length(deps)?)
        }
        QueryMsg::GetFundingPaymentHistory {
            user_address,
            index,
        } => to_binary(&get_funding_payment_history(deps, user_address, index)?),
        QueryMsg::GetFundingRateHistoryLength {} => {
            to_binary(&get_funding_rate_history_length(deps)?)
        }
        QueryMsg::GetFundingRateHistory { index } => {
            to_binary(&get_funding_rate_history(deps, index)?)
        }
        QueryMsg::GetLiquidationHistoryLength {} => {
            to_binary(&get_liquidation_history_length(deps)?)
        }
        QueryMsg::GetLiquidationHistory {
            user_address,
            index,
        } => to_binary(&get_liquidation_history(deps, user_address, index)?),
        QueryMsg::GetTradeHistoryLength {} => to_binary(&get_trade_history_length(deps)?),
        QueryMsg::GetTradeHistory {
            user_address,
            index,
        } => to_binary(&get_trade_history(deps, user_address, index)?),
        QueryMsg::GetMarketInfo { market_index } => {
            to_binary(&get_market_info(deps, market_index)?)
        }
    }
}

fn get_user(deps: Deps, user_address: String) -> StdResult<UserResponse> {
    let user = Users.load(
        deps.storage,
        &addr_validate_to_lower(deps.api, &user_address)?,
    )?;
    let ur = UserResponse {
        collateral: user.collateral,
        cumulative_deposits: user.cumulative_deposits,
        total_fee_paid: user.total_fee_paid,
        total_token_discount: user.total_token_discount,
        total_referral_reward: user.total_referral_reward,
        total_referee_discount: user.total_token_discount,
        positions_length: user.positions_length,
    };
    Ok(ur)
}

fn get_user_position(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<UserPositionResponse> {
    let position = Positions.load(
        deps.storage,
        (&addr_validate_to_lower(deps.api, &user_address)?, index),
    )?;
    let upr = UserPositionResponse {
        base_asset_amount: position.base_asset_amount,
        quote_asset_amount: position.quote_asset_amount,
        last_cumulative_funding_rate: position.last_cumulative_funding_rate,
        last_cumulative_repeg_rebate: position.last_cumulative_repeg_rebate,
        last_funding_rate_ts: position.last_funding_rate_ts,
        stop_loss_price: position.stop_loss_price,
        stop_loss_amount: position.stop_loss_amount,
        stop_profit_price: position.stop_profit_price,
        stop_profit_amount: position.stop_profit_amount,
        transfer_to: "DefaultAddress".to_string(),
    };
    Ok(upr)
}

fn get_admin(deps: Deps) -> StdResult<AdminResponse> {
    let state = STATE.load(deps.storage)?;
    let admin = AdminResponse {
        admin: state.admin.into(),
    };
    Ok(admin)
}

fn is_exchange_paused(deps: Deps) -> StdResult<IsExchangePausedResponse> {
    let state = STATE.load(deps.storage)?;
    let ex_paused = IsExchangePausedResponse {
        exchange_paused: state.exchange_paused,
    };
    Ok(ex_paused)
}

fn is_funding_paused(deps: Deps) -> StdResult<IsFundingPausedResponse> {
    let state = STATE.load(deps.storage)?;
    let funding_paused = IsFundingPausedResponse {
        funding_paused: state.funding_paused,
    };
    Ok(funding_paused)
}

fn admin_controls_prices(deps: Deps) -> StdResult<AdminControlsPricesResponse> {
    let state = STATE.load(deps.storage)?;
    let admin_control = AdminControlsPricesResponse {
        admin_controls_prices: state.admin_controls_prices,
    };
    Ok(admin_control)
}
fn get_vaults_address(deps: Deps) -> StdResult<VaultsResponse> {
    let state = STATE.load(deps.storage)?;
    let vaults = VaultsResponse {
        collateral_vault: state.collateral_vault.into(),
        insurance_vault: state.insurance_vault.into(),
    };
    Ok(vaults)
}
fn get_margin_ratios(deps: Deps) -> StdResult<MarginRatioResponse> {
    let state = STATE.load(deps.storage)?;
    let margin_ratio = MarginRatioResponse {
        margin_ratio_initial: state.margin_ratio_initial,
        margin_ratio_partial: state.margin_ratio_partial,
        margin_ratio_maintenance: state.margin_ratio_maintenance,
    };
    Ok(margin_ratio)
}
fn get_partial_liquidation_close_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidationClosePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liq_close_perc = PartialLiquidationClosePercentageResponse {
        numerator: state.partial_liquidation_close_percentage_numerator,
        denominator: state.partial_liquidation_close_percentage_denominator,
    };
    Ok(partial_liq_close_perc)
}
fn get_partial_liquidation_penalty_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidationPenaltyPercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liq_penalty_perc = PartialLiquidationPenaltyPercentageResponse {
        numerator: state.partial_liquidation_penalty_percentage_numerator,
        denominator: state.partial_liquidation_penalty_percentage_denominator,
    };
    Ok(partial_liq_penalty_perc)
}

fn get_full_liquidation_penalty_percentage(
    deps: Deps,
) -> StdResult<FullLiquidationPenaltyPercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let full_liq_penalty_perc = FullLiquidationPenaltyPercentageResponse {
        numerator: state.full_liquidation_penalty_percentage_numerator,
        denominator: state.full_liquidation_penalty_percentage_denominator,
    };
    Ok(full_liq_penalty_perc)
}

fn get_partial_liquidator_share_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidatorSharePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liquidator_share_perc = PartialLiquidatorSharePercentageResponse {
        denominator: state.partial_liquidation_liquidator_share_denominator,
    };
    Ok(partial_liquidator_share_perc)
}

fn get_full_liquidator_share_percentage(
    deps: Deps,
) -> StdResult<FullLiquidatorSharePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let full_liquidator_share_perc = FullLiquidatorSharePercentageResponse {
        denominator: state.full_liquidation_liquidator_share_denominator,
    };
    Ok(full_liquidator_share_perc)
}
fn get_max_deposit_limit(deps: Deps) -> StdResult<MaxDepositLimitResponse> {
    let state = STATE.load(deps.storage)?;
    let max_deposit = MaxDepositLimitResponse {
        max_deposit: state.max_deposit,
    };
    Ok(max_deposit)
}
fn get_fee_structure(deps: Deps) -> StdResult<FeeStructureResponse> {
    let state = STATE.load(deps.storage)?;
    let fee_structure = FeeStructureResponse {
        fee_numerator: state.fee_structure.fee_numerator,
        fee_denominator: state.fee_structure.fee_denominator,
        first_tier: state.fee_structure.first_tier,
        second_tier: state.fee_structure.second_tier,
        third_tier: state.fee_structure.third_tier,
        fourth_tier: state.fee_structure.fourth_tier,
        referrer_reward_numerator: state.fee_structure.referrer_reward_numerator,
        referrer_reward_denominator: state.fee_structure.referrer_reward_denominator,
        referee_discount_numerator: state.fee_structure.referee_discount_numerator,
        referee_discount_denominator: state.fee_structure.referee_discount_denominator,
    };
    Ok(fee_structure)
}

fn get_curve_history_length(deps: Deps) -> StdResult<CurveHistoryLengthResponse> {
    let state = CurveHistoryInfo.load(deps.storage)?;
    let length = CurveHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
fn get_curve_history(deps: Deps, index: u64) -> StdResult<CurveHistoryResponse> {
    let state = CurveHistory.load(deps.storage, index)?;
    let curve_history = CurveHistoryResponse {
        ts: state.ts,
        record_id: state.record_id,
        market_index: state.market_index,
        peg_multiplier_before: state.peg_multiplier_before,
        base_asset_reserve_before: state.base_asset_reserve_before,
        quote_asset_reserve_before: state.quote_asset_reserve_before,
        sqrt_k_before: state.sqrt_k_before,
        peg_multiplier_after: state.peg_multiplier_after,
        base_asset_reserve_after: state.base_asset_reserve_after,
        quote_asset_reserve_after: state.quote_asset_reserve_after,
        sqrt_k_after: state.sqrt_k_after,
        base_asset_amount_long: state.base_asset_amount_long,
        base_asset_amount_short: state.base_asset_amount_short,
        base_asset_amount: state.base_asset_amount,
        open_interest: state.open_interest,
        total_fee: state.total_fee,
        total_fee_minus_distributions: state.total_fee_minus_distributions,
        adjustment_cost: state.adjustment_cost,
        oracle_price: state.oracle_price,
        trade_record: state.trade_record,
    };
    Ok(curve_history)
}

fn get_deposit_history_length(deps: Deps) -> StdResult<DepositHistoryLengthResponse> {
    let state = DepositHistoryInfo.load(deps.storage)?;
    let length = DepositHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
fn get_deposit_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<DepositHistoryResponse> {
    let state = DepositHistory.load(
        deps.storage,
        (index, addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let deposit_history = DepositHistoryResponse {
        ts: state.ts,
        record_id: state.record_id,
        user: state.user.into(),
        direction: state.direction,
        collateral_before: state.collateral_before,
        cumulative_deposits_before: state.cumulative_deposits_before,
        amount: state.amount,
    };
    Ok(deposit_history)
}
fn get_funding_payment_history_length(
    deps: Deps,
) -> StdResult<FundingPaymentHistoryLengthResponse> {
    let state = FundingPaymentHistoryInfo.load(deps.storage)?;
    let length = FundingPaymentHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
fn get_funding_payment_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<FundingPaymentHistoryResponse> {
    let state = FundingPaymentHistory.load(
        deps.storage,
        (index, &addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let fp_history = FundingPaymentHistoryResponse {
        ts: state.ts,
        record_id: state.record_id,
        user: user_address,
        market_index: state.market_index,
        funding_payment: state.funding_payment,
        base_asset_amount: state.base_asset_amount,
        user_last_cumulative_funding: state.user_last_cumulative_funding,
        user_last_funding_rate_ts: state.user_last_funding_rate_ts,
        amm_cumulative_funding_long: state.amm_cumulative_funding_long,
        amm_cumulative_funding_short: state.amm_cumulative_funding_short,
    };
    Ok(fp_history)
}

fn get_funding_rate_history_length(deps: Deps) -> StdResult<FundingRateHistoryLengthResponse> {
    let state = FundingRateHistoryInfo.load(deps.storage)?;
    let length = FundingRateHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
fn get_funding_rate_history(deps: Deps, index: u64) -> StdResult<FundingRateHistoryResponse> {
    let state = FundingRateHistory.load(deps.storage, index)?;
    let fr_history = FundingRateHistoryResponse {
        ts: state.ts,
        record_id: state.record_id,
        market_index: state.market_index,
        funding_rate: state.funding_rate,
        cumulative_funding_rate_long: state.cumulative_funding_rate_long,
        cumulative_funding_rate_short: state.cumulative_funding_rate_short,
        oracle_price_twap: state.oracle_price_twap,
        mark_price_twap: state.mark_price_twap,
    };
    Ok(fr_history)
}

fn get_liquidation_history_length(deps: Deps) -> StdResult<LiquidationHistoryLengthResponse> {
    let state = LiquidationHistoryInfo.load(deps.storage)?;
    let length = LiquidationHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
fn get_liquidation_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<LiquidationHistoryResponse> {
    let state = LiquidationHistory.load(
        deps.storage,
        (index, addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let liq_history = LiquidationHistoryResponse {
        ts: state.ts,
        record_id: state.record_id,
        user: user_address,
        partial: state.partial,
        base_asset_value: state.base_asset_value,
        base_asset_value_closed: state.base_asset_value_closed,
        liquidation_fee: state.liquidation_fee,
        fee_to_liquidator: state.fee_to_liquidator,
        fee_to_insurance_fund: state.fee_to_insurance_fund,
        liquidator: state.liquidator.into(),
        total_collateral: state.total_collateral,
        collateral: state.collateral,
        unrealized_pnl: state.unrealized_pnl,
        margin_ratio: state.margin_ratio,
    };
    Ok(liq_history)
}

fn get_trade_history_length(deps: Deps) -> StdResult<TradeHistoryLengthResponse> {
    let state = TradeHistoryInfo.load(deps.storage)?;
    let length = TradeHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
fn get_trade_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<TradeHistoryResponse> {
    let state = TradeHistory.load(
        deps.storage,
        (index, addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let trade_history = TradeHistoryResponse {
        ts: state.ts,
        record_id: state.record_id,
        user: user_address,
        direction: state.direction,
        base_asset_amount: state.base_asset_amount,
        quote_asset_amount: state.quote_asset_amount,
        mark_price_before: state.mark_price_before,
        mark_price_after: state.mark_price_after,
        fee: state.fee,
        referrer_reward: state.referrer_reward,
        referee_discount: state.referee_discount,
        token_discount: state.token_discount,
        liquidation: state.liquidation,
        market_index: state.market_index,
        oracle_price: state.oracle_price,
    };
    Ok(trade_history)
}

fn get_market_info(deps: Deps, market_index: u64) -> StdResult<MarketInfoResponse> {
    let state = Markets.load(deps.storage, market_index)?;
    let market_info = MarketInfoResponse {
        market_name: state.market_name,
        initialized: state.initialized,
        base_asset_amount_long: state.base_asset_amount_long,
        base_asset_amount_short: state.base_asset_amount_short,
        base_asset_amount: state.base_asset_amount,
        open_interest: state.open_interest,
        oracle: state.amm.oracle.into(),
        oracle_source: state.amm.oracle_source,
        base_asset_reserve: state.amm.base_asset_reserve,
        quote_asset_reserve: state.amm.quote_asset_reserve,
        cumulative_repeg_rebate_long: state.amm.cumulative_repeg_rebate_long,
        cumulative_repeg_rebate_short: state.amm.cumulative_repeg_rebate_short,
        cumulative_funding_rate_long: state.amm.cumulative_funding_rate_long,
        cumulative_funding_rate_short: state.amm.cumulative_funding_rate_short,
        last_funding_rate: state.amm.last_funding_rate,
        last_funding_rate_ts: state.amm.last_funding_rate_ts,
        funding_period: state.amm.funding_period,
        last_oracle_price_twap: state.amm.last_oracle_price_twap,
        last_mark_price_twap: state.amm.last_mark_price_twap,
        last_mark_price_twap_ts: state.amm.last_mark_price_twap_ts,
        sqrt_k: state.amm.sqrt_k,
        peg_multiplier: state.amm.peg_multiplier,
        total_fee: state.amm.total_fee,
        total_fee_minus_distributions: state.amm.total_fee_minus_distributions,
        total_fee_withdrawn: state.amm.total_fee_withdrawn,
        minimum_trade_size: state.amm.minimum_trade_size,
        last_oracle_price_twap_ts: state.amm.last_oracle_price_twap_ts,
        last_oracle_price: state.amm.last_oracle_price,
    };
    Ok(market_info)
}

// response query template
// fn (deps: Deps) -> StdResult<> {
//     let state = STATE.load(deps.storage)?;
//     let  =  {
//     };
//     Ok()
// }
