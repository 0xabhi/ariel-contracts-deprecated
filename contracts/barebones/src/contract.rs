
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
    //TODO:: adding condition to check the initialization, if it's done already
    let fs = FeeStructure {
        fee_numerator: DEFAULT_FEE_NUMERATOR,
        fee_denominator: DEFAULT_FEE_DENOMINATOR,


        first_tier_minimum_balance: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_MINIMUM_BALANCE,
        first_tier_discount_numerator: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_DISCOUNT_NUMERATOR,
        first_tier_discount_denominator: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_DISCOUNT_DENOMINATOR,
    
        second_tier_minimum_balance: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_MINIMUM_BALANCE,
        second_tier_discount_numerator: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_DISCOUNT_NUMERATOR,
        second_tier_discount_denominator: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_DISCOUNT_DENOMINATOR,
    
        third_tier_minimum_balance: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_MINIMUM_BALANCE,
        third_tier_discount_numerator: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_DISCOUNT_NUMERATOR,
        third_tier_discount_denominator: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_DISCOUNT_DENOMINATOR,
    
        fourth_tier_minimum_balance: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_MINIMUM_BALANCE,
        fourth_tier_discount_numerator: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_DISCOUNT_NUMERATOR,
        fourth_tier_discount_denominator: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_DISCOUNT_DENOMINATOR,

        referrer_reward_numerator: DEFAULT_REFERRER_REWARD_NUMERATOR,
        referrer_reward_denominator: DEFAULT_REFERRER_REWARD_DENOMINATOR,
        referee_discount_numerator: DEFAULT_REFEREE_DISCOUNT_NUMERATOR,
        referee_discount_denominator: DEFAULT_REFEREE_DISCOUNT_DENOMINATOR,
    };
    let oracle_gr = OracleGuardRails {
        use_for_liquidations: true,
        mark_oracle_divergence_numerator: 1,
        mark_oracle_divergence_denominator: 10,
        slots_before_stale: 1000,
        confidence_interval_max_size: 4,
        too_volatile_ratio: 5,
    };
    let orderstate = OrderState {
        min_order_quote_asset_amount: 0,
        reward_numerator: 0,
        reward_denominator: 0,
        time_based_reward_lower_bound: 0, // minimum filler reward for time-based reward
    };
    let state = State {
        exchange_paused: false,
        funding_paused: false,
        admin_controls_prices: true,
        collateral_vault: addr_validate_to_lower(deps.api, &msg.collateral_vault).unwrap(),
        insurance_vault: addr_validate_to_lower(deps.api, &msg.insurance_vault).unwrap(),
        oracle: addr_validate_to_lower(deps.api, &msg.oracle)?,
        margin_ratio_initial: 2000u128,
        margin_ratio_maintenance: 500u128,
        margin_ratio_partial: 625u128,
        partial_liquidation_close_percentage_numerator: 25u128,
        partial_liquidation_close_percentage_denominator: 100u128,
        partial_liquidation_penalty_percentage_numerator: 25u128,
        partial_liquidation_penalty_percentage_denominator: 100u128,
        full_liquidation_penalty_percentage_numerator: 1u128,
        full_liquidation_penalty_percentage_denominator: 1u128,
        partial_liquidation_liquidator_share_denominator: 25u64,
        full_liquidation_liquidator_share_denominator: 2000u64,
        max_deposit: 0u128,
        markets_length: 0u64,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // ADMIN.set(deps.branch(), Some(info.sender.clone()))?;
    STATE.save(deps.storage, &state)?;
    // STATE.load(deps.storage)?;

    FEESTRUCTURE.save(deps.storage, &fs)?;
    ORACLEGUARDRAILS.save(deps.storage, &oracle_gr)?;
    ORDERSTATE.save(deps.storage, &orderstate)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.clone()))
}