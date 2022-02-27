use crate::{states::{state::CONFIG, user::{User, USER}}, msg::UserResponse};
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128, StdResult, Deps};

use crate::ContractError;

pub fn try_open_position(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn try_close_position(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new())
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
    USER.update(deps.storage, info.sender.clone().as_str(), &update_user)?;

    Ok(Response::new()
        .add_attribute("method", "deposit_collateral")
        .add_attribute("amount", amount))
}

pub fn try_withdraw_collateral(
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
    USER.update(deps.storage, info.sender.clone().as_str(), &update_user)?;
    Ok(Response::new()
        .add_attribute("method", "withdraw_collateral")
        .add_attribute("amount", amount))
}



pub fn get_user_data(deps: Deps, user_address: String) -> StdResult<UserResponse> {
    let user = USER.load(deps.storage, user_address.as_str())?;
    Ok(UserResponse {
        user_address: user.user_address.into(),
        free_collateral: user.free_collateral,
        total_deposits: user.total_deposits,
        total_paid_fees: user.total_paid_fees,
    })
}
