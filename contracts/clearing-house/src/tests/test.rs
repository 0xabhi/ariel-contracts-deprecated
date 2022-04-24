use crate::contract::{instantiate, query};
use crate::states::constants::{DEFAULT_FEE_DENOMINATOR, DEFAULT_FEE_NUMERATOR};
use crate::views::execute_admin::{
     try_initialize_market,
};

use crate::views::execute_user::{try_deposit_collateral, try_withdraw_collateral};

use ariel::execute::InstantiateMsg;
use ariel::queries::QueryMsg;
use ariel::response::*;

use ariel::types::{DepositDirection, OracleSource};
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    coins, from_binary, Decimal, Uint128,
};

const ADMIN_ACCOUNT: &str = "admin_account";

#[test]
pub fn test_initialize_state() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        collateral_vault: String::from(MOCK_CONTRACT_ADDR),
        insurance_vault: String::from(MOCK_CONTRACT_ADDR),
        admin_controls_prices: true,
        oracle: String::from(MOCK_CONTRACT_ADDR),
    };

    let info = mock_info(ADMIN_ACCOUNT, &coins(0, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAdmin {}).unwrap();
    // let value: AdminResponse = from_binary(&res).unwrap();
    // assert_eq!(String::from(ADMIN_ACCOUNT), value.admin);
    //query market_length
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMarketLength {});
    match res {
        Ok(_res) => {
            let value: MarketLengthResponse = from_binary(&_res).unwrap();
            assert_eq!(0, value.length);
        }
        Err(err) => {
            println!("{} days", err);
        }
    }
    //query vault address
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVaults {}).unwrap();
    let value: VaultsResponse = from_binary(&res).unwrap();
    assert_eq!(String::from(MOCK_CONTRACT_ADDR), value.collateral_vault); //collateral vault set
    assert_eq!(String::from(MOCK_CONTRACT_ADDR), value.insurance_vault); // insurance vault setassert_eq!(String::from(MOCK_CONTRACT_ADDR), value.insurance_vault); // insurance vault set
                                                                         // query admin

    //query isexchangepaused

    let res = query(deps.as_ref(), mock_env(), QueryMsg::IsExchangePaused {}).unwrap();
    let value: IsExchangePausedResponse = from_binary(&res).unwrap();
    assert_eq!(false, value.exchange_paused);

    //query funding paused

    let res = query(deps.as_ref(), mock_env(), QueryMsg::IsFundingPaused {}).unwrap();
    let value: IsFundingPausedResponse = from_binary(&res).unwrap();
    assert_eq!(false, value.funding_paused);
    //query admin controls prices

    let res = query(deps.as_ref(), mock_env(), QueryMsg::AdminControlsPrices {}).unwrap();
    let value: AdminControlsPricesResponse = from_binary(&res).unwrap();
    assert_eq!(true, value.admin_controls_prices);

    //query margin ratio

    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMarginRatio {}).unwrap();
    let value: MarginRatioResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(2000u128), value.margin_ratio_initial);
    assert_eq!(Uint128::from(500u128), value.margin_ratio_maintenance);
    assert_eq!(Uint128::from(625u128), value.margin_ratio_partial);

    //query oracle
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOracle {}).unwrap();
    let value: OracleResponse = from_binary(&res).unwrap();
    assert_eq!(MOCK_CONTRACT_ADDR, value.oracle);

    //query oracle guard rails
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOracleGuardRails {}).unwrap();
    let value: OracleGuardRailsResponse = from_binary(&res).unwrap();
    assert_eq!(true, value.use_for_liquidations);
    assert_eq!(1000, value.slots_before_stale.i128());

    // query order state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOrderState {}).unwrap();
    let value: OrderStateResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::zero(), value.min_order_quote_asset_amount);

    //query partial liq close
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPartialLiquidationClosePercentage {},
    )
    .unwrap();
    let value: PartialLiquidationClosePercentageResponse = from_binary(&res).unwrap();
    // 25/100
    assert_eq!(Decimal::percent(25), value.value);

    //query partial liq penalty
    //query full liq penalty
    //query partial liq share perc
    //query full liq share perc
    //query max deposit
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMaxDepositLimit {}).unwrap();
    let value: MaxDepositLimitResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::zero(), value.max_deposit);

    // query fee structure
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetFeeStructure {}).unwrap();
    let value: FeeStructureResponse = from_binary(&res).unwrap();
    assert_eq!(
        Decimal::from_ratio(DEFAULT_FEE_NUMERATOR, DEFAULT_FEE_DENOMINATOR),
        value.fee
    );
}

#[test]
pub fn test_deposit_withdraw() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        collateral_vault: String::from("collateral_vault"),
        insurance_vault: String::from("insurance_vault"),
        admin_controls_prices: true,
        oracle: String::from(MOCK_CONTRACT_ADDR),
    };

    let info = mock_info(ADMIN_ACCOUNT, &coins(0, "earth"));

    // we can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    // intialize market
    let market_init_info = mock_info(ADMIN_ACCOUNT, &coins(0, "earth"));
    let amm_base_asset_reserve = Uint128::from(5000_000_000u128);
    let amm_quote_asset_reserve = Uint128::from(5000_000_000u128);
    let amm_periodicity = 10;
    let oracle_source = OracleSource::Oracle;
    let amm_peg_multiplier = Uint128::from(92_19_0000u128);
    let margin_ratio_initial = 2000;
    let margin_ratio_partial = 625;
    let margin_ratio_maintenance = 500;
    let _err = try_initialize_market(
        deps.as_mut(),
        mock_env(),
        market_init_info,
        1,
        "LUNA-UST".to_string(),
        amm_base_asset_reserve,
        amm_quote_asset_reserve,
        amm_periodicity,
        amm_peg_multiplier,
        oracle_source,
        margin_ratio_initial,
        margin_ratio_partial,
        margin_ratio_maintenance,
    )
    .unwrap();

    // println!("{} contract error", err);
    //test market params
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetMarketInfo { market_index: 1 },
    )
    .unwrap();
    let value: MarketInfoResponse = from_binary(&res).unwrap();
    assert_eq!("LUNA-UST".to_string(), value.market_name);
    assert_eq!(amm_base_asset_reserve, value.sqrt_k);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMarketLength {}).unwrap();
    let value: MarketLengthResponse = from_binary(&res).unwrap();
    assert_eq!(1, value.length);

    let deposit_info = mock_info("geekybot", &coins(100_000_000, "uusd"));

    try_deposit_collateral(deps.as_mut(), mock_env(), deposit_info, 100_000_000, None).unwrap();
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUser {
            user_address: "geekybot".to_string(),
        },
    )
    .unwrap();
    let value: UserResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(100_000_000u128), value.collateral);
    assert_eq!(Uint128::zero(), value.total_fee_paid);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDepositHistoryLength {},
    )
    .unwrap();
    let value: DepositHistoryLengthResponse = from_binary(&res).unwrap();
    assert_eq!(1, value.length);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDepositHistory {
            user_address: "geekybot".to_string(),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let value: Vec<DepositHistoryResponse> = from_binary(&res).unwrap();
    assert_eq!(100000000, value[0].amount);
    assert_eq!(DepositDirection::DEPOSIT, value[0].direction);

    //test withdraw
    let withdraw_info = mock_info("geekybot", &coins(0, "uusd"));
    try_withdraw_collateral(deps.as_mut(), mock_env(), withdraw_info, 100_000_000).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetUser {
            user_address: "geekybot".to_string(),
        },
    )
    .unwrap();
    let value: UserResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(100_000_000u128), value.collateral);
    assert_eq!(Uint128::zero(), value.total_fee_paid);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDepositHistoryLength {},
    )
    .unwrap();
    let value: DepositHistoryLengthResponse = from_binary(&res).unwrap();
    assert_eq!(1, value.length);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDepositHistory {
            user_address: "geekybot".to_string(),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let value: Vec<DepositHistoryResponse> = from_binary(&res).unwrap();
    assert_eq!(100000000, value[0].amount);
    assert_eq!(DepositDirection::DEPOSIT, value[0].direction);
}
