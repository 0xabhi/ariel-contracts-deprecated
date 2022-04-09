use crate::error::ContractError;

use ariel::types::{OracleGuardRails, OraclePriceData, OracleSource, OracleStatus};
use cosmwasm_std::Addr;

use crate::helpers::amm;
use crate::states::market::Amm;

pub fn block_operation(
    a: &Amm,
    oracle_account_info: &Addr,
    clock_slot: u64,
    guard_rails: &OracleGuardRails,
    precomputed_mark_price: Option<u128>,
) -> Result<(bool, OraclePriceData), ContractError> {
    let OracleStatus {
        price_data: oracle_price_data,
        is_valid: oracle_is_valid,
        mark_too_divergent: is_oracle_mark_too_divergent,
        oracle_mark_spread_pct: _,
    } = get_oracle_status(
        a,
        oracle_account_info,
        clock_slot,
        guard_rails,
        precomputed_mark_price,
    )?;

    let block = !oracle_is_valid || is_oracle_mark_too_divergent;
    Ok((block, oracle_price_data))
}

pub fn get_oracle_data(
    a: &Amm,
    price_oracle: &Addr,
    clock_slot: u64,
) -> Result<(i128, i128, u128, u128, i64), ContractError> {
    // // TODO
    // // Get Oracle data
    // let pyth_price_data = price_oracle
    //     .try_borrow_data()
    //     .or(Err(ContractError::UnableToLoadOracle))?;
    // let price_data = pyth_client::cast::<pyth_client::Price>(&pyth_price_data);

    // let oracle_price = cast_to_i128(price_data.agg.price)?;
    // let oracle_conf = cast_to_u128(price_data.agg.conf)?;
    // let oracle_twap = cast_to_i128(price_data.twap.val)?;
    // let oracle_twac = cast_to_u128(price_data.twac.val)?;

    // let oracle_precision = 10_u128.pow(price_data.expo.unsigned_abs());

    // let mut oracle_scale_mult = 1;
    // let mut oracle_scale_div = 1;

    // if oracle_precision > MARK_PRICE_PRECISION {
    //     oracle_scale_div = oracle_precision
    //         .checked_div(MARK_PRICE_PRECISION).ok_or(Err(ContractError::MathError ));
    // } else {
    //     oracle_scale_mult = MARK_PRICE_PRECISION
    //         .checked_div(oracle_precision)?;
    // }

    // let oracle_price_scaled = (oracle_price)
    //     .checked_mul(cast(oracle_scale_mult)?)?
    //     .checked_div(cast(oracle_scale_div)?)?;

    // let oracle_twap_scaled = (oracle_twap)
    //     .checked_mul(cast(oracle_scale_mult)?)?
    //     .checked_div(cast(oracle_scale_div)?)?;

    // let oracle_conf_scaled = (oracle_conf)
    //     .checked_mul(oracle_scale_mult)?
    //     .checked_div(oracle_scale_div)?;

    // let oracle_twac_scaled = (oracle_twac)
    //     .checked_mul(oracle_scale_mult)?
    //     .checked_div(oracle_scale_div)?;

    // let oracle_delay: i64 = cast_to_i64(clock_slot)?
    //     .checked_sub(cast(price_data.valid_slot)?)?;

    // Ok((
    //     oracle_price_scaled,
    //     oracle_twap_scaled,
    //     oracle_conf_scaled,
    //     oracle_twac_scaled,
    //     oracle_delay,
    // ))
    Ok((0, 0, 0, 0, 0))
}

pub fn get_oracle_price(
    a: &Amm,
    oracle_account_info: &Addr,
    clock_slot: u64,
) -> Result<OraclePriceData, ContractError> {
    Ok(OraclePriceData {
        price: 0,
        confidence: 0,
        delay: 0,
        has_sufficient_number_of_data_points: true,
    })
}

pub fn get_switchboard_price(
    a: &Amm,
    oracle_account_info: &Addr,
    clock_slot: u64,
) -> Result<OraclePriceData, ContractError> {
    Ok(OraclePriceData {
        price: 0,
        confidence: 0,
        delay: 0,
        has_sufficient_number_of_data_points: true,
    })
}

pub fn get_oracle_status(
    a: &Amm,
    oracle_account_info: &Addr,
    clock_slot: u64,
    guard_rails: &OracleGuardRails,
    precomputed_mark_price: Option<u128>,
) -> Result<OracleStatus, ContractError> {
    let oracle_price_data = get_oracle_price(a, oracle_account_info, clock_slot)?;
    let oracle_is_valid = amm::is_oracle_valid(a, &oracle_price_data, &guard_rails)?;
    let oracle_mark_spread_pct =
        amm::calculate_oracle_mark_spread_pct(a, &oracle_price_data, precomputed_mark_price)?;
    let is_oracle_mark_too_divergent =
        amm::is_oracle_mark_too_divergent(oracle_mark_spread_pct, &guard_rails)?;

    Ok(OracleStatus {
        price_data: oracle_price_data,
        oracle_mark_spread_pct,
        is_valid: oracle_is_valid,
        mark_too_divergent: is_oracle_mark_too_divergent,
    })
}

//TODO get oracle price twap
pub fn get_oracle_twap(price_oracle: &Addr) -> Result<Option<i128>, ContractError> {
    //todo
    Ok(Some(0))
}
