use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw2::set_contract_version;
use cw_utils::maybe_addr;

use crate::helpers::constants::*;
use crate::states::order::OrderState;
use crate::states::state::{State, ADMIN, STATE};

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
        fee_numerator: DEFAULT_FEE_NUMERATOR,
        fee_denominator: DEFAULT_FEE_DENOMINATOR,
        first_tier: DiscountTokenTier {
            minimum_balance: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_MINIMUM_BALANCE,
            discount_numerator: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_DISCOUNT_NUMERATOR,
            discount_denominator: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_DISCOUNT_DENOMINATOR,
        },

        second_tier: DiscountTokenTier {
            minimum_balance: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_MINIMUM_BALANCE,
            discount_numerator: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_DISCOUNT_NUMERATOR,
            discount_denominator: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_DISCOUNT_DENOMINATOR,
        },
        third_tier: DiscountTokenTier {
            minimum_balance: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_MINIMUM_BALANCE,
            discount_numerator: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_DISCOUNT_NUMERATOR,
            discount_denominator: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_DISCOUNT_DENOMINATOR,
        },
        fourth_tier: DiscountTokenTier {
            minimum_balance: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_MINIMUM_BALANCE,
            discount_numerator: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_DISCOUNT_NUMERATOR,
            discount_denominator: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_DISCOUNT_DENOMINATOR,
        },
        referrer_reward_numerator: DEFAULT_REFERRER_REWARD_NUMERATOR,
        referrer_reward_denominator: DEFAULT_REFERRER_REWARD_DENOMINATOR,
        referee_discount_numerator: DEFAULT_REFEREE_DISCOUNT_NUMERATOR,
        referee_discount_denominator: DEFAULT_REFEREE_DISCOUNT_DENOMINATOR,
    };
    let oracle_gr = OracleGuardRails {
        use_for_liquidations: true,
        mark_oracle_divergence_numerator: 1,
        mark_oracle_divergence_denominator: 10,
        slots_before_stale: 1000,
        confidence_interval_max_size: 4,
        too_volatile_ratio: 5,
    };
    let orderstate = OrderState {
        min_order_quote_asset_amount: 0,
        reward_numerator: 0,
        reward_denominator: 0,
        time_based_reward_lower_bound: 0, // minimum filler reward for time-based reward
    };
    let state = State {
        exchange_paused: false,
        funding_paused: false,
        admin_controls_prices: true,
        collateral_vault: addr_validate_to_lower(deps.api, &msg.collateral_vault).unwrap(),
        insurance_vault: addr_validate_to_lower(deps.api, &msg.insurance_vault).unwrap(),
        oracle: addr_validate_to_lower(deps.api, &msg.oracle)?,
        margin_ratio_initial: 2000,
        margin_ratio_maintenance: 500,
        margin_ratio_partial: 625,
        partial_liquidation_close_percentage_numerator: 25,
        partial_liquidation_close_percentage_denominator: 100,
        partial_liquidation_penalty_percentage_numerator: 25,
        partial_liquidation_penalty_percentage_denominator: 100,
        full_liquidation_penalty_percentage_numerator: 1,
        full_liquidation_penalty_percentage_denominator: 1,
        partial_liquidation_liquidator_share_denominator: 2,
        full_liquidation_liquidator_share_denominator: 20,
        max_deposit: 0,
        fee_structure: fs,
        oracle_guard_rails: oracle_gr,
        markets_length: 0,
        orderstate,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender.clone()))?;
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
    let api = deps.api;
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
        },
        ExecuteMsg::WithdrawCollateral { amount } => {
            try_withdraw_collateral(deps, _env, info, amount)
        },
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
        ExecuteMsg::CancelOrder {
            market_index,
            order_id,
        } => try_cancel_order(deps, _env, info, market_index, order_id),
        ExecuteMsg::ExpireOrders {user_address} => try_expire_orders(deps, _env, info, user_address),
        ExecuteMsg::FillOrder { order_id, user_address, market_index } => try_fill_order(deps, _env, info, order_id, user_address, market_index),
        ExecuteMsg::ClosePosition { market_index } => {
            try_close_position(deps, _env, info, market_index)
        },
        ExecuteMsg::Liquidate { user, market_index } => {
            try_liquidate(deps, _env, info, user, market_index)
        },
        ExecuteMsg::MoveAMMPrice {
            base_asset_reserve,
            quote_asset_reserve,
            market_index,
        } => try_move_amm_price(deps, base_asset_reserve, quote_asset_reserve, market_index),
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
        } => try_repeg_amm_curve(deps, _env, new_peg_candidate, market_index),
        ExecuteMsg::UpdateAMMOracleTwap { market_index } => {
            try_update_amm_oracle_twap(deps, _env, market_index)
        },
        ExecuteMsg::ResetAMMOracleTwap { market_index } => {
            try_reset_amm_oracle_twap(deps, _env, market_index)
        },
        ExecuteMsg::SettleFundingPayment {} => try_settle_funding_payment(deps, _env, info),
        ExecuteMsg::UpdateFundingRate { market_index } => {
            try_update_funding_rate(deps, _env, market_index)
        },
        ExecuteMsg::UpdateK {
            market_index,
            sqrt_k,
        } => try_update_k(deps, _env, market_index, sqrt_k),
        ExecuteMsg::UpdateMarginRatio {
            market_index,
            margin_ratio_initial,
            margin_ratio_partial,
            margin_ratio_maintenance,
        } => try_update_margin_ratio(
            deps,
            info,
            market_index,
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
        },
        ExecuteMsg::UpdateFullLiquidationLiquidatorShareDenominator { denominator } => {
            try_update_full_liquidation_liquidator_share_denominator(deps, info, denominator)
        },
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
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin.into())?)?)
        },
        ExecuteMsg::UpdateMaxDeposit { max_deposit } => {
            try_update_max_deposit(deps, info, max_deposit)
        },
        ExecuteMsg::UpdateExchangePaused { exchange_paused } => {
            try_update_exchange_paused(deps, info, exchange_paused)
        },
        ExecuteMsg::DisableAdminControlsPrices {} => try_disable_admin_control_prices(deps, info),
        ExecuteMsg::UpdateFundingPaused { funding_paused } => {
            try_update_funding_paused(deps, info, funding_paused)
        },
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
        ExecuteMsg::UpdateOrderState {
            min_order_quote_asset_amount,
            reward_numerator,
            reward_denominator,
            time_based_reward_lower_bound,
        } => try_update_order_state_structure(
            deps,
            info,
            min_order_quote_asset_amount,
            reward_numerator,
            reward_denominator,
            time_based_reward_lower_bound,
        ),
        ExecuteMsg::UpdateMarketOracle {
            market_index,
            oracle,
            oracle_source,
        } => try_update_market_oracle(deps, info, market_index, oracle, oracle_source),
        ExecuteMsg::UpdateOracleAddress { oracle } => try_update_oracle_address(deps, info, oracle),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetUser { user_address } => Ok(to_binary(&get_user(deps, user_address)?)?),
        QueryMsg::GetUserMarketPosition {
            user_address,
            index,
        } => Ok(to_binary(&get_user_position(deps, user_address, index)?)?),
        QueryMsg::GetUserPositions{ user_address } => Ok(to_binary(&get_active_positions(deps, user_address)?)?),
        QueryMsg::GetAdmin {} => Ok(to_binary(&get_admin(deps)?)?),
        QueryMsg::IsExchangePaused {} => Ok(to_binary(&is_exchange_paused(deps)?)?),
        QueryMsg::IsFundingPaused {} => Ok(to_binary(&is_funding_paused(deps)?)?),
        QueryMsg::AdminControlsPrices {} => Ok(to_binary(&admin_controls_prices(deps)?)?),
        QueryMsg::GetVaults {} => Ok(to_binary(&get_vaults_address(deps)?)?),
        QueryMsg::GetMarginRatio {} => Ok(to_binary(&get_margin_ratios(deps)?)?),
        QueryMsg::GetPartialLiquidationClosePercentage {} => {
            Ok(to_binary(&get_partial_liquidation_close_percentage(deps)?)?)
        }
        QueryMsg::GetPartialLiquidationPenaltyPercentage {} => {
            Ok(to_binary(&get_partial_liquidation_penalty_percentage(deps)?)?)
        }
        QueryMsg::GetFullLiquidationPenaltyPercentage {} => {
            Ok(to_binary(&get_full_liquidation_penalty_percentage(deps)?)?)
        }
        QueryMsg::GetPartialLiquidatorSharePercentage {} => {
            Ok(to_binary(&get_partial_liquidator_share_percentage(deps)?)?)
        }
        QueryMsg::GetFullLiquidatorSharePercentage {} => {
            Ok(to_binary(&get_full_liquidator_share_percentage(deps)?)?)
        }
        QueryMsg::GetMaxDepositLimit {} => Ok(to_binary(&get_max_deposit_limit(deps)?)?),
        QueryMsg::GetOracle{} => Ok(to_binary(&get_oracle_address(deps)?)?),
        QueryMsg::GetMarketLength{} => Ok(to_binary(&get_market_length(deps)?)?),
        QueryMsg::GetOracleGuardRails{} => Ok(to_binary(&get_oracle_guard_rails(deps)?)?),
        QueryMsg::GetOrderState{} => Ok(to_binary(&get_order_state(deps)?)?),
        QueryMsg::GetFeeStructure {} => Ok(to_binary(&get_fee_structure(deps)?)?),
        QueryMsg::GetCurveHistoryLength {} => Ok(to_binary(&get_curve_history_length(deps)?)?),
        QueryMsg::GetCurveHistory { index } => Ok(to_binary(&get_curve_history(deps, index)?)?),
        QueryMsg::GetDepositHistoryLength {} => Ok(to_binary(&get_deposit_history_length(deps)?)?),
        QueryMsg::GetDepositHistory {
            user_address,
            index,
        } => Ok(to_binary(&get_deposit_history(deps, user_address, index)?)?),
        QueryMsg::GetFundingPaymentHistoryLength {} => {
            Ok(to_binary(&get_funding_payment_history_length(deps)?)?)
        }
        QueryMsg::GetFundingPaymentHistory {
            user_address,
            index,
        } => Ok(to_binary(&get_funding_payment_history(deps, user_address, index)?)?),
        QueryMsg::GetFundingRateHistoryLength {} => {
            Ok(to_binary(&get_funding_rate_history_length(deps)?)?)
        }
        QueryMsg::GetFundingRateHistory { index } => {
            Ok(to_binary(&get_funding_rate_history(deps, index)?)?)
        }
        QueryMsg::GetLiquidationHistoryLength {} => {
            Ok(to_binary(&get_liquidation_history_length(deps)?)?)
        }
        QueryMsg::GetLiquidationHistory {
            user_address,
            index,
        } => Ok(to_binary(&get_liquidation_history(deps, user_address, index)?)?),
        QueryMsg::GetTradeHistoryLength {} => Ok(to_binary(&get_trade_history_length(deps)?)?),
        QueryMsg::GetTradeHistory {
            user_address,
            index,
        } => Ok(to_binary(&get_trade_history(deps, user_address, index)?)?),
        QueryMsg::GetMarketInfo { market_index } => {
            Ok(to_binary(&get_market_info(deps, market_index)?)?)
        }
    }
}
