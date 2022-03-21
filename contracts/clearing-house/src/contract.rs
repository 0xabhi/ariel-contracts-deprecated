#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::states::state::{State, STATE};

use ariel::execute::{ExecuteMsg, InstantiateMsg};
use ariel::queries::QueryMsg;
use ariel::response::*;
use ariel::types::PositionDirection;

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
    let state = State {
        admin: info.sender.clone(),
        collateral_vault: deps.api.addr_validate(&msg.collateral_vault())?,
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
            amm_base_asset_reserve,
            amm_quote_asset_reserve,
            amm_periodicity,
            amm_peg_multiplier,
        } => try_initialize_market(
            deps,
            info,
            market_index,
            amm_base_asset_reserve,
            amm_quote_asset_reserve,
            amm_periodicity,
            amm_peg_multiplier,
        ),
        ExecuteMsg::DepositCollateral { amount } => try_deposit_collateral(deps, info, amount),
        ExecuteMsg::WithdrawCollateral { amount } => try_withdraw_collateral(deps, info, amount),
        ExecuteMsg::OpenPosition {
            direction,
            quote_asset_amount,
            market_index,
            limit_price,
        } => try_open_position(
            deps,
            info,
            direction,
            quote_asset_amount,
            market_index,
            limit_price,
        ),
        ExecuteMsg::ClosePosition { market_index } => try_close_position(deps, info, market_index),
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
        } => try_repeg_amm_curve(deps, info, new_peg_candidate, market_index),
        ExecuteMsg::SettleFundingPayment {} => try_settle_funding_payment(deps, info),
        ExecuteMsg::UpdateFundingRate { market_index } => {
            try_update_funding_rate(deps, info, market_index)
        }
        ExecuteMsg::UpdateK {
            market_index,
            sqrt_k,
        } => try_update_k(deps, info, market_index, sqrt_k),
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
            t1_minimum_balance,
            t1_discount_numerator,
            t1_discount_denominator,
            t2_minimum_balance,
            t2_discount_numerator,
            t2_discount_denominator,
            t3_minimum_balance,
            t3_discount_numerator,
            t3_discount_denominator,
            referrer_reward_numerator,
            referrer_reward_denominator,
            referee_discount_numerator,
            referee_discount_denominator,
        } => try_update_fee(
            deps,
            info,
            fee_numerator,
            fee_denominator,
            t1_minimum_balance,
            t1_discount_numerator,
            t1_discount_denominator,
            t2_minimum_balance,
            t2_discount_numerator,
            t2_discount_denominator,
            t3_minimum_balance,
            t3_discount_numerator,
            t3_discount_denominator,
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
    info: MessageInfo,
    market_index: u64,
    amm_base_asset_reserve: u128,
    amm_quote_asset_reserve: u128,
    amm_periodicity: u128,
    amm_peg_multiplier: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_initialize_market"))
}

pub fn try_deposit_collateral(
    deps: DepsMut,
    info: MessageInfo,
    amount: i128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_deposit_collateral"))
}

pub fn try_withdraw_collateral(
    deps: DepsMut,
    info: MessageInfo,
    amount: i128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_withdraw_collateral"))
}

pub fn try_open_position(
    deps: DepsMut,
    info: MessageInfo,
    direction: PositionDirection,
    quote_asset_amount: u128,
    market_index: u64,
    limit_price: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_open_position"))
}

pub fn try_close_position(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
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
    Ok(Response::new().add_attribute("method", "try_move_amm_price"))
}

pub fn try_withdraw_fees(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    amount: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_withdraw_fees"))
}

pub fn try_withdraw_from_insurance_vault_to_market(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    amount: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_withdraw_from_insurance_vault_to_market"))
}

pub fn try_repeg_amm_curve(
    deps: DepsMut,
    info: MessageInfo,
    new_peg_candidate: u128,
    market_index: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_repeg_amm_curve"))
}

pub fn try_settle_funding_payment(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_settle_funding_payment"))
}
pub fn try_update_funding_rate(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_funding_rate"))
}

pub fn try_update_k(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    sqrt_k: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_k"))
}

pub fn try_update_market_minimum_trade_size(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    minimum_trade_size: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_market_minimum_trade_size"))
}

pub fn try_update_margin_ratio(
    deps: DepsMut,
    info: MessageInfo,
    margin_ratio_initial: u128,
    margin_ratio_partial: u128,
    margin_ratio_maintenance: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_margin_ratio"))
}

pub fn try_update_partial_liquidation_close_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_partial_liquidation_close_percentage"))
}

pub fn try_update_partial_liquidation_penalty_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128,
) -> Result<Response, ContractError> {
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
    Ok(Response::new().add_attribute("method", "try_update_full_liquidation_penalty_percentage"))
}

pub fn try_update_partial_liquidation_liquidator_share_denominator(
    deps: DepsMut,
    info: MessageInfo,
    denominator: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute(
        "method",
        "try_update_partial_liquidation_liquidator_share_denominator",
    ))
}

pub fn try_update_full_liquidation_liquidator_share_denominator(
    deps: DepsMut,
    info: MessageInfo,
    denominator: u128,
) -> Result<Response, ContractError> {
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
    t1_minimum_balance: u64,
    t1_discount_numerator: u128,
    t1_discount_denominator: u128,

    t2_minimum_balance: u64,
    t2_discount_numerator: u128,
    t2_discount_denominator: u128,

    t3_minimum_balance: u64,
    t3_discount_numerator: u128,
    t3_discount_denominator: u128,

    referrer_reward_numerator: u128,
    referrer_reward_denominator: u128,
    referee_discount_numerator: u128,
    referee_discount_denominator: u128,
) -> Result<Response, ContractError> {
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
    Ok(Response::new().add_attribute("method", "try_update_oracle_guard_rails"))
}
pub fn try_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_admin"))
}

pub fn try_update_max_deposit(
    deps: DepsMut,
    info: MessageInfo,
    max_deposit: u128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_max_deposit"))
}

pub fn try_update_exchange_paused(
    deps: DepsMut,
    info: MessageInfo,
    exchange_paused: bool,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_exchange_paused"))
}

pub fn try_disable_admin_control_prices(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_disable_admin_control_prices"))
}
pub fn try_update_funding_paused(
    deps: DepsMut,
    info: MessageInfo,
    funding_paused: bool,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_funding_paused"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUser { user_address } => to_binary(&get_user(deps, user_address)?),
        QueryMsg::GetUserMarketPosition {
            user_address,
            index,
        } => to_binary(&get_user_position(
            deps: Deps,
            user_address: String,
            index: u64,
        )?),
        QueryMsg::GetAdmin {} => to_binary(&get_admin(deps: Deps)?),
        QueryMsg::IsExchangePaused {} => to_binary(&is_exchange_paused(deps: Deps)?),
        QueryMsg::IsFundingPaused {} => to_binary(&is_funding_paused(deps: Deps)?),
        QueryMsg::AdminControlsPrices {} => to_binary(&admin_controls_prices(deps: Deps)?),
        QueryMsg::GetVaults {} => to_binary(&get_vaults_address(deps: Deps)?),
        QueryMsg::GetMarginRatio {} => to_binary(&get_margin_ratios(deps: Deps)?),
        QueryMsg::GetPartialLiquidationClosePercentage {} => {
            to_binary(&get_partial_liquidation_close_percentage(deps: Deps)?)
        }
        QueryMsg::GetPartialLiquidationPenaltyPercentage {} => {
            to_binary(&get_partial_liquidation_penalty_percentage(deps: Deps)?)
        }
        QueryMsg::GetFullLiquidationPenaltyPercentage {} => {
            to_binary(&get_full_liquidation_penalty_percentage(deps: Deps)?)
        }
        QueryMsg::GetPartialLiquidatorSharePercentage {} => {
            to_binary(&get_partial_liquidator_share_percentage(deps: Deps)?)
        }
        QueryMsg::GetFullLiquidatorSharePercentage {} => {
            to_binary(&get_full_liquidator_share_percentage(deps: Deps)?)
        }
        QueryMsg::GetMaxDepositLimit {} => to_binary(&get_max_deposit_limit(deps: Deps)?),
        QueryMsg::GetFeeStructure {} => to_binary(&get_fee_structure(deps: Deps)?),
        QueryMsg::GetCurveHistoryLength {} => to_binary(&get_curve_history_length(deps: Deps)?),
        QueryMsg::GetCurveHistory { index } => {
            to_binary(&get_curve_history(deps: Deps, index: u64)?)
        }
        QueryMsg::GetDepositHistoryLength {} => to_binary(&get_deposit_history_length(deps: Deps)?),
        QueryMsg::GetDepositHistory { index } => {
            to_binary(&get_deposit_history(deps: Deps, index: u64)?)
        }
        QueryMsg::GetFundingPaymentHistoryLength {} => {
            to_binary(&get_funding_payment_history_length(deps: Deps)?)
        }
        QueryMsg::GetFundingPaymentHistory { index } => {
            to_binary(&get_funding_payment_history(deps: Deps, index: u64)?)
        }
        QueryMsg::GetFundingRateHistoryLength {} => {
            to_binary(&get_funding_rate_history_length(deps: Deps)?)
        }
        QueryMsg::GetFundingRateHistory { index } => {
            to_binary(&get_funding_rate_history(deps: Deps, index: u64)?)
        }
        QueryMsg::GetLiquidationHistoryLength {} => {
            to_binary(&get_liquidation_history_length(deps: Deps)?)
        }
        QueryMsg::GetLiquidationHistory { index } => {
            to_binary(&get_liquidation_history(deps: Deps, index: u64)?)
        }
        QueryMsg::GetTradeHistoryLength {} => to_binary(&get_trade_history_length(deps: Deps)?),
        QueryMsg::GetTradeHistory { index } => {
            to_binary(&get_trade_history(deps: Deps, index: u64)?)
        }
        QueryMsg::GetMarketInfo { market_index } => {
            to_binary(&get_market_info(deps: Deps, market_index: u64)?)
        }
    }
}

fn get_user(deps: Deps, user_address: String) -> StdResult<UserResponse> {
    let state = STATE.load(deps.storage)?;
    let ur = UserResponse {
        collateral: 125,
        cumulative_deposits: 250,
        total_fee_paid: 10,
        total_token_discount: 5,
        total_referral_reward: 2,
        total_referee_discount: 3,
        positions_length: 2,
    };
    Ok(ur)
}

fn get_user_position(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<UserPositionResponse> {
    let state = STATE.load(deps.storage)?;
    let upr = UserPositionResponse {};
    Ok(upr)
}

fn get_admin(deps: Deps) -> StdResult<AdminResponse> {
    let state = STATE.load(deps.storage)?;
    let admin = AdminResponse {};
    Ok(admin)
}

fn is_exchange_paused(deps: Deps) -> StdResult<IsExchangePausedResponse> {
    let state = STATE.load(deps.storage)?;
    let ex_paused = IsExchangePausedResponse {};
    Ok(ex_paused)
}

fn is_funding_paused(deps: Deps) -> StdResult<IsFundingPausedResponse> {
    let state = STATE.load(deps.storage)?;
    let funding_paused = IsFundingPausedResponse {};
    Ok(funding_paused)
}

fn admin_controls_prices(deps: Deps) -> StdResult<AdminControlsPricesResponse> {
    let state = STATE.load(deps.storage)?;
    let admin_control = AdminControlsPricesResponse {};
    Ok(admin_control)
}
fn get_vaults_address(deps: Deps) -> StdResult<VaultsResponse> {
    let state = STATE.load(deps.storage)?;
    let vaults = VaultsResponse {};
    Ok(vaults)
}
fn get_margin_ratios(deps: Deps) -> StdResult<MarginRatioResponse> {
    let state = STATE.load(deps.storage)?;
    let margin_ratio = MarginRatioResponse {};
    Ok(margin_ratio)
}
fn get_partial_liquidation_close_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidationClosePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liq_close_perc = PartialLiquidationClosePercentageResponse {};
    Ok(partial_liq_close_perc)
}
fn get_partial_liquidation_penalty_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidationPenaltyPercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liq_penalty_perc = PartialLiquidationPenaltyPercentageResponse {};
    Ok(partial_liq_penalty_perc)
}

fn get_full_liquidation_penalty_percentage(
    deps: Deps,
) -> StdResult<FullLiquidationPenaltyPercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let full_liq_penalty_perc = FullLiquidationPenaltyPercentageResponse {};
    Ok(full_liq_penalty_perc)
}

fn get_partial_liquidator_share_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidatorSharePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liquidator_share_perc = PartialLiquidatorSharePercentageResponse {};
    Ok(partial_liquidator_share_perc)
}

fn get_full_liquidator_share_percentage(
    deps: Deps,
) -> StdResult<FullLiquidatorSharePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let full_liquidator_share_perc = FullLiquidatorSharePercentageResponse {};
    Ok(full_liquidator_share_perc)
}
fn get_max_deposit_limit(deps: Deps) -> StdResult<MaxDepositLimitResponse> {
    let state = STATE.load(deps.storage)?;
    let max_deposit = MaxDepositLimitResponse {};
    Ok(max_deposit)
}
fn get_fee_structure(deps: Deps) -> StdResult<FeeStructureResponse> {
    let state = STATE.load(deps.storage)?;
    let fee_structure = FeeStructureResponse {};
    Ok(fee_structure)
}

fn get_curve_history_length(deps: Deps) -> StdResult<CurveHistoryLengthResponse> {
    let state = STATE.load(deps.storage)?;
    let length = CurveHistoryLengthResponse {};
    Ok(length)
}
fn get_curve_history(deps: Deps, index: u64) -> StdResult<CurveHistoryResponse> {
    let state = STATE.load(deps.storage)?;
    let curve_history = CurveHistoryResponse {};
    Ok(curve_history)
}

fn get_deposit_history_length(deps: Deps) -> StdResult<DepositHistoryLengthResponse> {
    let state = STATE.load(deps.storage)?;
    let length = DepositHistoryLengthResponse {};
    Ok(length)
}
fn get_deposit_history(deps: Deps, index: u64) -> StdResult<DepositHistoryResponse> {
    let state = STATE.load(deps.storage)?;
    let depoist_history = DepositHistoryResponse {};
    Ok(deposit_history)
}
fn get_funding_payment_history_length(
    deps: Deps,
) -> StdResult<FundingPaymentHistoryLengthResponse> {
    let state = STATE.load(deps.storage)?;
    let length = FundingPaymentHistoryLengthResponse {};
    Ok(length)
}
fn get_funding_payment_history(deps: Deps, index: u64) -> StdResult<FundingPaymentHistoryResponse> {
    let state = STATE.load(deps.storage)?;
    let fp_history = FundingPaymentHistoryResponse {};
    Ok(fp_history)
}

fn get_funding_rate_history_length(deps: Deps) -> StdResult<FundingRateHistoryLengthResponse> {
    let state = STATE.load(deps.storage)?;
    let length = FundingRateHistoryLengthResponse {};
    Ok(length)
}
fn get_funding_rate_history(deps: Deps, index: u64) -> StdResult<FundingRateHistoryResponse> {
    let state = STATE.load(deps.storage)?;
    let fr_history = FundingRateHistoryResponse {};
    Ok(fr_history)
}

fn get_liquidation_history_length(deps: Deps) -> StdResult<LiquidationHistoryLengthResponse> {
    let state = STATE.load(deps.storage)?;
    let length = LiquidationHistoryLengthResponse {};
    Ok(length)
}
fn get_liquidation_history(deps: Deps, index: u64) -> StdResult<LiquidationHistoryResponse> {
    let state = STATE.load(deps.storage)?;
    let liq_history = LiquidationHistoryResponse {};
    Ok(liq_history)
}

fn get_trade_history_length(deps: Deps) -> StdResult<TradeHistoryLengthResponse> {
    let state = STATE.load(deps.storage)?;
    let length = TradeHistoryLengthResponse {};
    Ok(length)
}
fn get_trade_history(deps: Deps, index: u64) -> StdResult<TradeHistoryResponse> {
    let state = STATE.load(deps.storage)?;
    let trade_history = TradeHistoryResponse {};
    Ok(trade_history)
}

fn get_market_info(deps: Deps, market_index: u64) -> StdResult<MarketInfoResponse> {
    let state = STATE.load(deps.storage)?;
    let market_info = MarketInfoResponse {};
    Ok(market_info)
}

// response query template
// fn (deps: Deps) -> StdResult<> {
//     let state = STATE.load(deps.storage)?;
//     let  =  {
//     };
//     Ok()
// }
