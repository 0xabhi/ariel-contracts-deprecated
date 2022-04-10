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
        transfer_to: "DefaultAddress".to_string(),
        stop_loss_price: todo!(),
        stop_loss_amount: todo!(),
        stop_profit_price: todo!(),
        stop_profit_amount: todo!(),
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
    let fs = STATE.load(deps.storage)?;
    let fee_structure = FeeStructureResponse {
        fee_numerator: fs.fee_structure.fee_numerator,
        fee_denominator: fs.fee_structure.fee_denominator,
        first_tier: fs.fee_structure.first_tier,
        second_tier: fs.fee_structure.second_tier,
        third_tier: fs.fee_structure.third_tier,
        fourth_tier: fs.fee_structure.fourth_tier,
        referrer_reward_numerator: fs.fee_structure.referrer_reward_numerator,
        referrer_reward_denominator: fs.fee_structure.referrer_reward_denominator,
        referee_discount_numerator: fs.fee_structure.referee_discount_numerator,
        referee_discount_denominator: fs.fee_structure.referee_discount_denominator,
    };
    Ok(fee_structure)
}

pub fn get_curve_history_length(deps: Deps) -> StdResult<CurveHistoryLengthResponse> {
    let ch_info = CurveHistoryInfo.load(deps.storage)?;
    let length = CurveHistoryLengthResponse {
        length: ch_info.len as u64,
    };
    Ok(length)
}
pub fn get_curve_history(deps: Deps, index: u64) -> StdResult<CurveHistoryResponse> {
    let ch = CurveHistory.load(deps.storage, index)?;
    let curve_history = CurveHistoryResponse {
        ts: ch.ts,
        record_id: ch.record_id,
        market_index: ch.market_index,
        peg_multiplier_before: ch.peg_multiplier_before,
        base_asset_reserve_before: ch.base_asset_reserve_before,
        quote_asset_reserve_before: ch.quote_asset_reserve_before,
        sqrt_k_before: ch.sqrt_k_before,
        peg_multiplier_after: ch.peg_multiplier_after,
        base_asset_reserve_after: ch.base_asset_reserve_after,
        quote_asset_reserve_after: ch.quote_asset_reserve_after,
        sqrt_k_after: ch.sqrt_k_after,
        base_asset_amount_long: ch.base_asset_amount_long,
        base_asset_amount_short: ch.base_asset_amount_short,
        base_asset_amount: ch.base_asset_amount,
        open_interest: ch.open_interest,
        total_fee: ch.total_fee,
        total_fee_minus_distributions: ch.total_fee_minus_distributions,
        adjustment_cost: ch.adjustment_cost,
        oracle_price: ch.oracle_price,
        trade_record: ch.trade_record,
    };
    Ok(curve_history)
}

pub fn get_deposit_history_length(deps: Deps) -> StdResult<DepositHistoryLengthResponse> {
    let dh_history = DepositHistoryInfo.load(deps.storage)?;
    let length = DepositHistoryLengthResponse {
        length: dh_history.len as u64,
    };
    Ok(length)
}
pub fn get_deposit_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<DepositHistoryResponse> {
    let dh = DepositHistory.load(
        deps.storage,
        (index, addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let deposit_history = DepositHistoryResponse {
        ts: dh.ts,
        record_id: dh.record_id,
        user: dh.user.into(),
        direction: dh.direction,
        collateral_before: dh.collateral_before,
        cumulative_deposits_before: dh.cumulative_deposits_before,
        amount: dh.amount,
    };
    Ok(deposit_history)
}
pub fn get_funding_payment_history_length(
    deps: Deps,
) -> StdResult<FundingPaymentHistoryLengthResponse> {
    let fp_info = FundingPaymentHistoryInfo.load(deps.storage)?;
    let length = FundingPaymentHistoryLengthResponse {
        length: fp_info.len as u64,
    };
    Ok(length)
}
pub fn get_funding_payment_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<FundingPaymentHistoryResponse> {
    let fp = FundingPaymentHistory.load(
        deps.storage,
        (index, &addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let fp_history = FundingPaymentHistoryResponse {
        ts: fp.ts,
        record_id: fp.record_id,
        user: user_address,
        market_index: fp.market_index,
        funding_payment: fp.funding_payment,
        base_asset_amount: fp.base_asset_amount,
        user_last_cumulative_funding: fp.user_last_cumulative_funding,
        user_last_funding_rate_ts: fp.user_last_funding_rate_ts,
        amm_cumulative_funding_long: fp.amm_cumulative_funding_long,
        amm_cumulative_funding_short: fp.amm_cumulative_funding_short,
    };
    Ok(fp_history)
}

pub fn get_funding_rate_history_length(deps: Deps) -> StdResult<FundingRateHistoryLengthResponse> {
    let fr_info = FundingRateHistoryInfo.load(deps.storage)?;
    let length = FundingRateHistoryLengthResponse {
        length: fr_info.len as u64,
    };
    Ok(length)
}
pub fn get_funding_rate_history(deps: Deps, index: u64) -> StdResult<FundingRateHistoryResponse> {
    let fr = FundingRateHistory.load(deps.storage, index)?;
    let fr_history = FundingRateHistoryResponse {
        ts: fr.ts,
        record_id: fr.record_id,
        market_index: fr.market_index,
        funding_rate: fr.funding_rate,
        cumulative_funding_rate_long: fr.cumulative_funding_rate_long,
        cumulative_funding_rate_short: fr.cumulative_funding_rate_short,
        oracle_price_twap: fr.oracle_price_twap,
        mark_price_twap: fr.mark_price_twap,
    };
    Ok(fr_history)
}

pub fn get_liquidation_history_length(deps: Deps) -> StdResult<LiquidationHistoryLengthResponse> {
    let lh_info = LiquidationHistoryInfo.load(deps.storage)?;
    let length = LiquidationHistoryLengthResponse {
        length: lh_info.len as u64,
    };
    Ok(length)
}
pub fn get_liquidation_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<LiquidationHistoryResponse> {
    let lh = LiquidationHistory.load(
        deps.storage,
        (index, addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let liq_history = LiquidationHistoryResponse {
        ts: lh.ts,
        record_id: lh.record_id,
        user: user_address,
        partial: lh.partial,
        base_asset_value: lh.base_asset_value,
        base_asset_value_closed: lh.base_asset_value_closed,
        liquidation_fee: lh.liquidation_fee,
        fee_to_liquidator: lh.fee_to_liquidator,
        fee_to_insurance_fund: lh.fee_to_insurance_fund,
        liquidator: lh.liquidator.into(),
        total_collateral: lh.total_collateral,
        collateral: lh.collateral,
        unrealized_pnl: lh.unrealized_pnl,
        margin_ratio: lh.margin_ratio,
    };
    Ok(liq_history)
}

pub fn get_trade_history_length(deps: Deps) -> StdResult<TradeHistoryLengthResponse> {
    let th_info = TradeHistoryInfo.load(deps.storage)?;
    let length = TradeHistoryLengthResponse {
        length: th_info.len as u64,
    };
    Ok(length)
}
pub fn get_trade_history(
    deps: Deps,
    user_address: String,
    index: u64,
) -> StdResult<TradeHistoryResponse> {
    let th = TradeHistory.load(
        deps.storage,
        (index, addr_validate_to_lower(deps.api, &user_address)?),
    )?;
    let trade_history = TradeHistoryResponse {
        ts: th.ts,
        record_id: th.record_id,
        user: user_address,
        direction: th.direction,
        base_asset_amount: th.base_asset_amount,
        quote_asset_amount: th.quote_asset_amount,
        mark_price_before: th.mark_price_before,
        mark_price_after: th.mark_price_after,
        fee: th.fee,
        referrer_reward: th.referrer_reward,
        referee_discount: th.referee_discount,
        token_discount: th.token_discount,
        liquidation: th.liquidation,
        market_index: th.market_index,
        oracle_price: th.oracle_price,
    };
    Ok(trade_history)
}

pub fn get_market_info(deps: Deps, market_index: u64) -> StdResult<MarketInfoResponse> {
    let market = Markets.load(deps.storage, market_index)?;
    let market_info = MarketInfoResponse {
        market_name: market.market_name,
        initialized: market.initialized,
        base_asset_amount_long: market.base_asset_amount_long,
        base_asset_amount_short: market.base_asset_amount_short,
        base_asset_amount: market.base_asset_amount,
        open_interest: market.open_interest,
        oracle: market.amm.oracle.into(),
        oracle_source: market.amm.oracle_source,
        base_asset_reserve: market.amm.base_asset_reserve,
        quote_asset_reserve: market.amm.quote_asset_reserve,
        cumulative_repeg_rebate_long: market.amm.cumulative_repeg_rebate_long,
        cumulative_repeg_rebate_short: market.amm.cumulative_repeg_rebate_short,
        cumulative_funding_rate_long: market.amm.cumulative_funding_rate_long,
        cumulative_funding_rate_short: market.amm.cumulative_funding_rate_short,
        last_funding_rate: market.amm.last_funding_rate,
        last_funding_rate_ts: market.amm.last_funding_rate_ts,
        funding_period: market.amm.funding_period,
        last_oracle_price_twap: market.amm.last_oracle_price_twap,
        last_mark_price_twap: market.amm.last_mark_price_twap,
        last_mark_price_twap_ts: market.amm.last_mark_price_twap_ts,
        sqrt_k: market.amm.sqrt_k,
        peg_multiplier: market.amm.peg_multiplier,
        total_fee: market.amm.total_fee,
        total_fee_minus_distributions: market.amm.total_fee_minus_distributions,
        total_fee_withdrawn: market.amm.total_fee_withdrawn,
        minimum_trade_size: todo!(),
        last_oracle_price_twap_ts: market.amm.last_oracle_price_twap_ts,
        last_oracle_price: market.amm.last_oracle_price,
    };
    Ok(market_info)
}
