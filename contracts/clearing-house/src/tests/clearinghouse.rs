use crate::contract::{execute, instantiate, query};
use crate::helpers::constants::{
    DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_DISCOUNT_DENOMINATOR,
    DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_MINIMUM_BALANCE, DEFAULT_FEE_NUMERATOR,
};
use crate::views::execute::*;
use crate::views::query::*;
use crate::ContractError;
use ariel::execute::{ExecuteMsg, InstantiateMsg};
use ariel::queries::QueryMsg;
use ariel::response::*;
use ariel::types::*;
use cosmwasm_std::testing::{
    mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{coins, from_binary};

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
    assert_eq!(false, value.admin_controls_prices);

    //query margin ratio

    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMarginRatio {}).unwrap();
    let value: MarginRatioResponse = from_binary(&res).unwrap();
    assert_eq!(2000, value.margin_ratio_initial);
    assert_eq!(500, value.margin_ratio_maintenance);
    assert_eq!(625, value.margin_ratio_partial);

    //query oracle
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOracle {}).unwrap();
    let value: OracleResponse = from_binary(&res).unwrap();
    assert_eq!(ADMIN_ACCOUNT, value.oracle);

    //query oracle guard rails
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOracleGuardRails {}).unwrap();
    let value: OracleGuardRailsResponse = from_binary(&res).unwrap();
    assert_eq!(true, value.use_for_liquidations);
    assert_eq!(1000, value.slots_before_stale);

    // query order state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOrderState {}).unwrap();
    let value: OrderStateResponse = from_binary(&res).unwrap();
    assert_eq!(0, value.min_order_quote_asset_amount);

    //query partial liq close
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPartialLiquidationClosePercentage {},
    )
    .unwrap();
    let value: PartialLiquidationClosePercentageResponse = from_binary(&res).unwrap();
    assert_eq!(25, value.numerator);
    assert_eq!(100, value.denominator);

    //query partial liq penalty
    //query full liq penalty
    //query partial liq share perc
    //query full liq share perc
    //query max deposit
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMaxDepositLimit {}).unwrap();
    let value: MaxDepositLimitResponse = from_binary(&res).unwrap();
    assert_eq!(0, value.max_deposit);

    // query fee structure
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetFeeStructure {}).unwrap();
    let value: FeeStructureResponse = from_binary(&res).unwrap();
    assert_eq!(DEFAULT_FEE_NUMERATOR, value.fee_numerator);
    assert_eq!(
        DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_MINIMUM_BALANCE,
        value.first_tier_minimum_balance
    );
}
