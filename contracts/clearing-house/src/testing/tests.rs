use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, UserResponse};
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Decimal256, Uint128};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let msg = InstantiateMsg {
        leverage: Uint128::new(5),
        collateral_fund: String::from("collateral_vault"),
        insurance_vault: String::from("insurance_vault"),
        initial_margin_ratio: Decimal256::from_atomics(20u64, 0).unwrap(),
        maintenance_margin_ratio: Decimal256::from_atomics(65u64, 1).unwrap(),
        liquidation_penalty: Decimal256::from_atomics(15u64, 1).unwrap(),
        liquidator_reward: Decimal256::from_atomics(1u64, 0).unwrap(),
        fee_percentage: Decimal256::from_atomics(3u64, 2).unwrap(),
        max_deposit: Uint128::new(100_000_000_000),
    };
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("5", value.leverage.to_string()); //test leverage
    assert_eq!(String::from("collateral_vault"), value.collateral_fund); //collateral vault set 
    assert_eq!(String::from("insurance_vault"), value.insurance_vault);  // insurance vault set 
    assert_eq!("20", value.initial_margin_ratio.to_string()); //test initial margin ratio
    assert_eq!("6.5", value.maintenance_margin_ratio.to_string()); //test mmr 6.5
    assert_eq!("1.5", value.liquidation_penalty.to_string()); //test liq pen 1.5
    assert_eq!("1", value.liquidator_reward.to_string()); //test liqudator reward 1
    assert_eq!("0.03", value.fee_percentage.to_string()); //test fee perce 0.03%
    assert_eq!("100000000000", value.max_deposit.to_string()); //test max deposit 1_000_000
        
}

#[test]
fn deposit_collateral_test() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let msg = InstantiateMsg {
        leverage: Uint128::new(5),
        collateral_fund: String::from("collateral_vault"),
        insurance_vault: String::from("insurance_vault"),
        initial_margin_ratio: Decimal256::from_atomics(20u64, 0).unwrap(),
        maintenance_margin_ratio: Decimal256::from_atomics(65u64, 1).unwrap(),
        liquidation_penalty: Decimal256::from_atomics(15u64, 1).unwrap(),
        liquidator_reward: Decimal256::from_atomics(1u64, 0).unwrap(),
        fee_percentage: Decimal256::from_atomics(3u64, 2).unwrap(),
        max_deposit: Uint128::new(100_000_000_000),
    };
    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // beneficiary can release it
    let info = mock_info("anyone", &coins(2, "token"));
    let dep_amount = Uint128::new(5_00000);
    let msg = ExecuteMsg::DepositCollateral { amount: dep_amount };
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
