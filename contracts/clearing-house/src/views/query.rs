// use crate::helpers::casting::cast_to_i64;
use crate::states::curve_history::*;
use crate::states::liquidation_history::{LiquidationHistory, LiquidationHistoryInfo};
use crate::states::market::Markets;
use crate::states::state::{STATE, ADMIN};
use crate::states::trade_history::{TradeHistory, TradeHistoryInfo};
use crate::states::user::{Positions, Users};
use crate::states::{deposit_history::*, funding_history::*};

use ariel::helper::addr_validate_to_lower;

use ariel::response::*;

use cosmwasm_std::{Deps, StdResult};

pub fn get_user(deps: Deps, user_address: String) -> StdResult<UserResponse> {
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

pub fn get_user_position(
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

pub fn get_admin(deps: Deps) -> StdResult<AdminResponse> {
    let state = STATE.load(deps.storage)?;
    let admin = AdminResponse {
        admin: ADMIN.query_admin(deps).unwrap().admin.unwrap(),
    };
    Ok(admin)
}

pub fn is_exchange_paused(deps: Deps) -> StdResult<IsExchangePausedResponse> {
    let state = STATE.load(deps.storage)?;
    let ex_paused = IsExchangePausedResponse {
        exchange_paused: state.exchange_paused,
    };
    Ok(ex_paused)
}

pub fn is_funding_paused(deps: Deps) -> StdResult<IsFundingPausedResponse> {
    let state = STATE.load(deps.storage)?;
    let funding_paused = IsFundingPausedResponse {
        funding_paused: state.funding_paused,
    };
    Ok(funding_paused)
}

pub fn admin_controls_prices(deps: Deps) -> StdResult<AdminControlsPricesResponse> {
    let state = STATE.load(deps.storage)?;
    let admin_control = AdminControlsPricesResponse {
        admin_controls_prices: state.admin_controls_prices,
    };
    Ok(admin_control)
}
pub fn get_vaults_address(deps: Deps) -> StdResult<VaultsResponse> {
    let state = STATE.load(deps.storage)?;
    let vaults = VaultsResponse {
        collateral_vault: state.collateral_vault.into(),
        insurance_vault: state.insurance_vault.into(),
    };
    Ok(vaults)
}
pub fn get_margin_ratios(deps: Deps) -> StdResult<MarginRatioResponse> {
    let state = STATE.load(deps.storage)?;
    let margin_ratio = MarginRatioResponse {
        margin_ratio_initial: state.margin_ratio_initial,
        margin_ratio_partial: state.margin_ratio_partial,
        margin_ratio_maintenance: state.margin_ratio_maintenance,
    };
    Ok(margin_ratio)
}
pub fn get_partial_liquidation_close_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidationClosePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liq_close_perc = PartialLiquidationClosePercentageResponse {
        numerator: state.partial_liquidation_close_percentage_numerator,
        denominator: state.partial_liquidation_close_percentage_denominator,
    };
    Ok(partial_liq_close_perc)
}
pub fn get_partial_liquidation_penalty_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidationPenaltyPercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liq_penalty_perc = PartialLiquidationPenaltyPercentageResponse {
        numerator: state.partial_liquidation_penalty_percentage_numerator,
        denominator: state.partial_liquidation_penalty_percentage_denominator,
    };
    Ok(partial_liq_penalty_perc)
}

pub fn get_full_liquidation_penalty_percentage(
    deps: Deps,
) -> StdResult<FullLiquidationPenaltyPercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let full_liq_penalty_perc = FullLiquidationPenaltyPercentageResponse {
        numerator: state.full_liquidation_penalty_percentage_numerator,
        denominator: state.full_liquidation_penalty_percentage_denominator,
    };
    Ok(full_liq_penalty_perc)
}

pub fn get_partial_liquidator_share_percentage(
    deps: Deps,
) -> StdResult<PartialLiquidatorSharePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let partial_liquidator_share_perc = PartialLiquidatorSharePercentageResponse {
        denominator: state.partial_liquidation_liquidator_share_denominator,
    };
    Ok(partial_liquidator_share_perc)
}

pub fn get_full_liquidator_share_percentage(
    deps: Deps,
) -> StdResult<FullLiquidatorSharePercentageResponse> {
    let state = STATE.load(deps.storage)?;
    let full_liquidator_share_perc = FullLiquidatorSharePercentageResponse {
        denominator: state.full_liquidation_liquidator_share_denominator,
    };
    Ok(full_liquidator_share_perc)
}
pub fn get_max_deposit_limit(deps: Deps) -> StdResult<MaxDepositLimitResponse> {
    let state = STATE.load(deps.storage)?;
    let max_deposit = MaxDepositLimitResponse {
        max_deposit: state.max_deposit,
    };
    Ok(max_deposit)
}
pub fn get_fee_structure(deps: Deps) -> StdResult<FeeStructureResponse> {
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

pub fn get_curve_history_length(deps: Deps) -> StdResult<CurveHistoryLengthResponse> {
    let state = CurveHistoryInfo.load(deps.storage)?;
    let length = CurveHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
pub fn get_curve_history(deps: Deps, index: u64) -> StdResult<CurveHistoryResponse> {
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

pub fn get_deposit_history_length(deps: Deps) -> StdResult<DepositHistoryLengthResponse> {
    let state = DepositHistoryInfo.load(deps.storage)?;
    let length = DepositHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
pub fn get_deposit_history(
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
pub fn get_funding_payment_history_length(
    deps: Deps,
) -> StdResult<FundingPaymentHistoryLengthResponse> {
    let state = FundingPaymentHistoryInfo.load(deps.storage)?;
    let length = FundingPaymentHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
pub fn get_funding_payment_history(
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

pub fn get_funding_rate_history_length(deps: Deps) -> StdResult<FundingRateHistoryLengthResponse> {
    let state = FundingRateHistoryInfo.load(deps.storage)?;
    let length = FundingRateHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
pub fn get_funding_rate_history(deps: Deps, index: u64) -> StdResult<FundingRateHistoryResponse> {
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

pub fn get_liquidation_history_length(deps: Deps) -> StdResult<LiquidationHistoryLengthResponse> {
    let state = LiquidationHistoryInfo.load(deps.storage)?;
    let length = LiquidationHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
pub fn get_liquidation_history(
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

pub fn get_trade_history_length(deps: Deps) -> StdResult<TradeHistoryLengthResponse> {
    let state = TradeHistoryInfo.load(deps.storage)?;
    let length = TradeHistoryLengthResponse {
        length: state.len as u64,
    };
    Ok(length)
}
pub fn get_trade_history(
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

pub fn get_market_info(deps: Deps, market_index: u64) -> StdResult<MarketInfoResponse> {
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
        minimum_trade_size: todo!(),
        last_oracle_price_twap_ts: state.amm.last_oracle_price_twap_ts,
        last_oracle_price: state.amm.last_oracle_price,
    };
    Ok(market_info)
}
