
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
}