#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::states::state::{State, STATE};
use crate::states::state::PositionDirection;

use ariel::execute::{ExecuteMsg, InstantiateMsg};
use ariel::queries::QueryMsg;
use ariel::response::*;

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
    amount: u128,
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
    limit_price: u128
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
    amount: u64
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_withdraw_fees"))
}


pub fn try_withdraw_from_insurance_vault(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    amount: u64
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_withdraw_from_insurance_vault"))
}

pub fn try_repeg_amm_curve(
    deps: DepsMut,
    info: MessageInfo,
    new_peg_candidate: u128,
    market_index: u64
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_repeg_amm_curve"))
}

pub fn try_settle_funding_payment(
    deps: DepsMut,
    info: MessageInfo
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_settle_funding_payment"))
}
pub fn try_update_funding_rate(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_funding_rate"))
}

pub fn try_update_k(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    sqrt_k: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_k"))
}

pub fn try_update_market_minimum_trade_size(
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    minimum_trade_size: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_market_minimum_trade_size"))
}

pub fn try_update_margin_ratio(
    deps: DepsMut,
    info: MessageInfo,
    margin_ratio_initial: u128,
    margin_ratio_partial: u128,
    margin_ratio_maintenance: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_margin_ratio"))
}

pub fn try_update_partial_liquidation_close_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_partial_liquidation_close_percentage"))
}


pub fn try_update_partial_liquidation_penalty_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_partial_liquidation_penalty_percentage"))
}


pub fn try_update_full_liquidation_penalty_percentage(
    deps: DepsMut,
    info: MessageInfo,
    numerator: u128,
    denominator: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_full_liquidation_penalty_percentage"))
}

pub fn try_update_partial_liquidation_liquidator_share_denominator(
    deps: DepsMut,
    info: MessageInfo,
    denominator: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_partial_liquidation_liquidator_share_denominator"))
}

pub fn try_update_full_liquidation_liquidator_share_denominator(
    deps: DepsMut,
    info: MessageInfo,
    denominator: u128
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "try_update_full_liquidation_liquidator_share_denominator"))
}


pub fn (
    deps: DepsMut,
    info: MessageInfo,
    market_index: u64,
    amount: u64
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", ""))
}







#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

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
}
