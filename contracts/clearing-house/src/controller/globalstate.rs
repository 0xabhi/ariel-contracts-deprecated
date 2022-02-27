use crate::states::state::CONFIG;
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128, Decimal256, StdResult, Deps};

use crate::{ContractError, msg::ConfigResponse};


pub fn try_update_collateral_vault(
    deps: DepsMut,
    info: MessageInfo,
    new_vault: String,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn try_update_insurance_vault(
    deps: DepsMut,
    info: MessageInfo,
    new_vault: String,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn try_update_margin_ratio(
    deps: DepsMut,
    info: MessageInfo,
    initial_mr: Decimal256,
    maintenance_mr: Decimal256,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn try_update_liquidation_config(
    deps: DepsMut,
    info: MessageInfo,
    liquidation_penalty: Decimal256,
    liquidation_reward: Decimal256,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn try_update_fee_percentage(
    deps: DepsMut,
    info: MessageInfo,
    new_fee: Decimal256,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn try_update_max_deposit(
    deps: DepsMut,
    info: MessageInfo,
    max_deposit: Decimal256,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn get_config_data(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.admin.into(),
        leverage: config.leverage,
        collateral_fund: config.collateral_fund.into(),
        insurance_vault: config.insurance_vault.into(),
        initial_margin_ratio: config.initial_margin_ratio,
        maintenance_margin_ratio: config.maintenance_margin_ratio,
        liquidation_penalty: config.liquidation_penalty,
        liquidator_reward: config.liquidator_reward,
        fee_percentage: config.fee_percentage,
        max_deposit: config.max_deposit,
    })
}