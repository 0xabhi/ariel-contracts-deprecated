#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Addr
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, UserResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, User, CONFIG, USER};

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
    let config = Config {
        owner: info.sender.clone(),
        leverage: msg.leverage,
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
    let config = CONFIG.load(deps.storage)?;
    let update_user  = |existing_user: Option<User>| -> StdResult<User> {
        match existing_user {
            None => Ok(User{
                user_address: info.sender.clone(),
                free_collateral: amount * config.leverage,
                total_deposits: amount,
                total_paid_fees: Uint128::new(0),
            }),
            Some(one) => Ok(User{
                user_address: info.sender.clone(),
                free_collateral: one.free_collateral * config.leverage,
                total_deposits: one.total_deposits + amount,
                total_paid_fees: one.total_paid_fees,    
            }),
        }
    };
    USER.update(deps.storage, &info.sender, &update_user)?;

    Ok(Response::new()
        .add_attribute("method", "deposit_collateral")
        .add_attribute("amount", amount)
    )
}

pub fn withdraw_collateral(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {

    let config = CONFIG.load(deps.storage)?;
    let update_user  = |existing_user: Option<User>| -> StdResult<User> {
        match existing_user {
            None => Ok(User{
                user_address: info.sender.clone(),
                free_collateral: amount * config.leverage,
                total_deposits: amount,
                total_paid_fees: Uint128::new(0),
            }),
            Some(one) => Ok(User{
                user_address: info.sender.clone(),
                free_collateral: one.free_collateral * config.leverage,
                total_deposits: one.total_deposits + amount,
                total_paid_fees: one.total_paid_fees,    
            }),
        }
    };
    // pseudo check for existing users and all before proceeding 
    USER.update(deps.storage, &info.sender, &update_user)?;
    Ok(Response::new()
        .add_attribute("method", "withdraw_collateral")
        .add_attribute("amount", amount)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUser { user_address } => to_binary(&get_user_data(deps, user_address)?),
        QueryMsg::GetConfig { } => to_binary(&get_config_data(deps)?),
    }
}

fn get_user_data(deps: Deps, user_address: Addr) -> StdResult<UserResponse> {
    let user = USER.load(deps.storage, &user_address)?;
    Ok(UserResponse { 
        user_address: user_address,
        free_collateral: user.free_collateral,
        total_deposits: user.total_deposits,
        total_paid_fees: user.total_paid_fees,
    })
}

fn get_config_data(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { owner: config.owner, leverage: config.leverage })
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { leverage: Uint128::new(5) };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
//         let value: ConfigResponse = from_binary(&res).unwrap();
//         assert_eq!(Uint128::new(5), value.leverage);
//     }
/*
    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
    */
// }
