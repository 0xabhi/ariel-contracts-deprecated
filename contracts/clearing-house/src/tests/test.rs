// use crate::contract::{execute, instantiate, query};
// use crate::views::execute::*;
// use crate::views::query::*;
// use ariel::execute::{ExecuteMsg, InstantiateMsg};
// use ariel::queries::QueryMsg;
// use ariel::response::*;
// use ariel::types::*;
// use cosmwasm_std::testing::{
//     mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info, MOCK_CONTRACT_ADDR,
// };
// use cosmwasm_std::{coins, from_binary};

// #[test]
// fn proper_initialization() {
//     let mut deps = mock_dependencies();

//     let msg = InstantiateMsg {
//         collateral_vault: String::from(MOCK_CONTRACT_ADDR),
//         insurance_vault: String::from(MOCK_CONTRACT_ADDR),
//         admin_controls_prices: true,
//     };

//     let info = mock_info("creator", &coins(0, "earth"));

//     // we can just call .unwrap() to assert this was a success
//     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//     assert_eq!(0, res.messages.len());

//     // it worked, let's query the state
//     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVaults {}).unwrap();
//     let value: VaultsResponse = from_binary(&res).unwrap();
//     assert_eq!(String::from(MOCK_CONTRACT_ADDR), value.collateral_vault); //collateral vault set
//     assert_eq!(String::from(MOCK_CONTRACT_ADDR), value.insurance_vault); // insurance vault set
// }

// #[test]
// fn proper_market_initialize() {
//     let mut deps = mock_dependencies();
//     let msg = InstantiateMsg {
//         collateral_vault: String::from(MOCK_CONTRACT_ADDR),
//         insurance_vault: String::from(MOCK_CONTRACT_ADDR),
//         admin_controls_prices: true,
//     };

//     let info = mock_info("creator", &coins(0, "earth"));
//     // we can just call .unwrap() to assert this was a success
//     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//     let dep_info = mock_info("testaddr", &coins(1000000, "uusd"));
//     let _env = mock_env();
//     let oracle_source = OracleSource::Simulated;
//     try_initialize_market(
//         deps.as_mut(),
//         _env,
//         info,
//         1,
//         "LUNA-UST".to_string(),
//         100,
//         100,
//         3600,
//         5,
//         oracle_source,
//         10,
//         6,
//         4,
//     )
//     .unwrap();
//     let res = query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::GetMarketInfo { market_index: 1 },
//     )
//     .unwrap();
//     let value: MarketInfoResponse = from_binary(&res).unwrap();
    
// }
