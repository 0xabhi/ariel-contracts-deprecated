use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw2::set_contract_version;

use crate::states::state::{State, STATE};

use ariel::execute::{ExecuteMsg, InstantiateMsg};
use ariel::helper::addr_validate_to_lower;
use ariel::queries::QueryMsg;

use ariel::types::{DiscountTokenTier, FeeStructure, OracleGuardRails};

use crate::error::ContractError;

use crate::views::{execute::*, query::*};

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
            oracle_source,
            margin_ratio_initial,
            margin_ratio_partial,
            margin_ratio_maintenance,
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
            oracle_source,
            margin_ratio_initial,
            margin_ratio_partial,
            margin_ratio_maintenance,
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
        ExecuteMsg::PlaceOrder { order } => try_place_order(deps, _env, info, order),
        ExecuteMsg::CancelOrder { order_id } => try_cancel_order(deps, _env, info, order_id),
        ExecuteMsg::CancelOrderByUserId { user_order_id } => {
            try_cancel_order_by_user_id(deps, _env, info, user_order_id)
        }
        ExecuteMsg::ExpireOrders {} => try_expire_orders(deps, _env, info),
        ExecuteMsg::FillOrder { order_id } => try_fill_order(deps, _env, info, order_id),
        ExecuteMsg::PlaceAndFillOrder { order } => {
            try_place_and_fill_order(deps, _env, info, order)
        }
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
        ExecuteMsg::UpdateAMMOracleTwap { market_index } => {
            try_update_amm_oracle_twap(deps, _env, info, market_index)
        }
        ExecuteMsg::ResetAMMOracleTwap { market_index } => {
            try_reset_amm_oracle_twap(deps, _env, info, market_index)
        }
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
        ExecuteMsg::UpdateMarketMinimumQuoteAssetTradeSize {
            market_index,
            minimum_trade_size,
        } => try_update_market_minimum_quote_asset_trade_size(
            deps,
            info,
            market_index,
            minimum_trade_size,
        ),
        ExecuteMsg::UpdateMarketMinimumBaseAssetTradeSize {
            market_index,
            minimum_trade_size,
        } => try_update_market_minimum_base_asset_trade_size(
            deps,
            info,
            market_index,
            minimum_trade_size,
        ),
        ExecuteMsg::UpdateOrderFillerRewardSystem {
            reward_numerator,
            reward_denominator,
            time_based_reward_lower_bound,
        } => try_update_order_filler_reward_structure(
            deps,
            info,
            reward_numerator,
            reward_denominator,
            time_based_reward_lower_bound,
        ),
        ExecuteMsg::UpdateMarketOracle {
            market_index,
            oracle,
            oracle_source
        } => try_update_market_oracle(deps, info, market_index, oracle, oracle_source)
    }
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

// response query template
// fn (deps: Deps) -> StdResult<> {
//     let state = STATE.load(deps.storage)?;
//     let  =  {
//     };
//     Ok()
// }
