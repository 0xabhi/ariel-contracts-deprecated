use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, UserResponse};
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Uint128};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let msg = InstantiateMsg {
        leverage: Uint128::new(5),
    };
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::new(5), value.leverage);
}

#[test]
fn deposit_collateral_test() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let msg = InstantiateMsg {
        leverage: Uint128::new(5),
    };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // beneficiary can release it
    let info = mock_info("anyone", &coins(2, "token"));
    let dep_amount = Uint128::new(5_00000);
    let msg = ExecuteMsg::DepositCollateral {
        amount: dep_amount,
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // should increase counter by 1
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUser {
            user_address: String::from("anyone"),
        },
    )
    .unwrap();
    let value: UserResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::new(25_00000), value.free_collateral);
    assert_eq!(Uint128::new(5_00000), value.total_deposits);

    // deposit again
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // should increase counter by 1
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUser {
            user_address: String::from("anyone"),
        },
    )
    .unwrap();
    
    let value: UserResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::new(50_00000), value.free_collateral);
    assert_eq!(Uint128::new(10_00000), value.total_deposits);

}
