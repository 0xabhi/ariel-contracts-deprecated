#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, UserResponse};
use crate::states::state::{Config, CONFIG, Market};
use crate::states::user::{User, USER};

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
        max_deposit: msg.max_deposit
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
        ExecuteMsg::WithdrawCollateral { amount } => withdraw_collateral(deps, info, amount),
    }
}

pub fn try_deposit_collateral(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // pseudo: Check if entered amount and sent amount of tokens are of same quantity and UST
    let config = CONFIG.load(deps.storage)?;
    let update_user = |existing_user: Option<User>| -> StdResult<User> {
        match existing_user {
            None => Ok(User {
                user_address: info.sender.clone(),
                free_collateral: amount * config.leverage,
                total_deposits: amount,
                total_paid_fees: Uint128::new(0),
            }),
            Some(one) => Ok(User {
                user_address: info.sender.clone(),
                free_collateral: one.free_collateral + (amount * config.leverage),
                total_deposits: one.total_deposits + amount,
                total_paid_fees: one.total_paid_fees,
            }),
        }
    };
    USER.update(deps.storage, info.sender.clone().into(), &update_user)?;

    Ok(Response::new()
        .add_attribute("method", "deposit_collateral")
        .add_attribute("amount", amount))
}

pub fn withdraw_collateral(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Pseudo::
    // Check if user exists or else throw error
    // check if he has enoug amount free for withdrawl
    let config = CONFIG.load(deps.storage)?;
    let update_user = |existing_user: Option<User>| -> StdResult<User> {
        match existing_user {
            None => Ok(User {
                user_address: info.sender.clone(),
                free_collateral: amount * config.leverage,
                total_deposits: amount,
                total_paid_fees: Uint128::new(0),
            }),
            Some(one) => Ok(User {
                user_address: info.sender.clone(),
                free_collateral: one.free_collateral * config.leverage,
                total_deposits: one.total_deposits,
                total_paid_fees: one.total_paid_fees,
            }),
        }
    };
    // pseudo check for existing users and all before proceeding
    USER.update(deps.storage, info.sender.clone().into(), &update_user)?;
    Ok(Response::new()
        .add_attribute("method", "withdraw_collateral")
        .add_attribute("amount", amount))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUser { user_address } => to_binary(&get_user_data(deps, user_address)?),
        QueryMsg::GetConfig {} => to_binary(&get_config_data(deps)?),
    }
}

fn get_user_data(deps: Deps, user_address: String) -> StdResult<UserResponse> {
    let user = USER.load(deps.storage, user_address)?;
    Ok(UserResponse {
        user_address: user.user_address.into(),
        free_collateral: user.free_collateral,
        total_deposits: user.total_deposits,
        total_paid_fees: user.total_paid_fees,
    })
}

fn get_config_data(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.admin.into(),
        leverage: config.leverage,
    })
}
