use crate::controller::globalstate::get_config_data;
#[cfg(not(feature = "library"))]
use crate::controller::user::get_user_data;
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::controller::globalstate::{
    try_update_collateral_vault, try_update_fee_percentage, try_update_insurance_vault,
    try_update_liquidation_config, try_update_margin_ratio, try_update_max_deposit,
};
use crate::controller::market::try_add_market;
use crate::controller::user::{
    try_close_position, try_deposit_collateral, try_open_position, try_withdraw_collateral,
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::states::state::{Config, CONFIG};

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
    let collateral_fund = deps.api.addr_validate(&msg.collateral_fund)?;
    let insurance_vault = deps.api.addr_validate(&msg.insurance_vault)?;

    let config = Config {
        admin: info.sender.clone(),
        leverage: msg.leverage,
        trade_paused: true,
        deposit_paused: true,
        price_controlled_by_admin: true,
        collateral_fund: collateral_fund,
        insurance_vault: insurance_vault,
        initial_margin_ratio: msg.initial_margin_ratio,
        maintenance_margin_ratio: msg.maintenance_margin_ratio,
        liquidation_penalty: msg.liquidation_penalty,
        liquidator_reward: msg.liquidator_reward,
        fee_percentage: msg.fee_percentage,
        max_deposit: msg.max_deposit,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("leverage", msg.leverage))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DepositCollateral { amount } => try_deposit_collateral(deps, info, amount),
        ExecuteMsg::WithdrawCollateral { amount } => try_withdraw_collateral(deps, info, amount),
        ExecuteMsg::UpdateCollateralVault { vault } => {
            try_update_collateral_vault(deps, info, vault)
        }
        ExecuteMsg::UpdateInsuranceVault { vault } => try_update_insurance_vault(deps, info, vault),
        ExecuteMsg::UpdateMarginRatio {
            initial_mr,
            maintenance_mr,
        } => try_update_margin_ratio(deps, info, initial_mr, maintenance_mr),
        ExecuteMsg::UpdateLiquidationConfig {
            liquidation_penalty,
            liquidation_reward,
        } => try_update_liquidation_config(deps, info, liquidation_penalty, liquidation_reward),
        ExecuteMsg::UpdateFeePercentage { new_fee } => {
            try_update_fee_percentage(deps, info, new_fee)
        }
        ExecuteMsg::UpdateMaxDeposit { max_deposit } => {
            try_update_max_deposit(deps, info, max_deposit)
        }
        ExecuteMsg::AddMarket {
            v_amm,
            long_base_asset_amount,
            short_base_asset_amount,
        } => try_add_market(
            deps,
            info,
            v_amm,
            long_base_asset_amount,
            short_base_asset_amount,
        ),
        ExecuteMsg::ClosePosition { market_index } => try_close_position(deps, info, market_index),

        ExecuteMsg::OpenPosition { market_index } => try_open_position(deps, info, market_index),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUser { user_address } => to_binary(&get_user_data(deps, user_address)?),
        QueryMsg::GetConfig {} => to_binary(&get_config_data(deps)?),
    }
}
