use cosmwasm_std::{Addr};

use crate::error::ContractError;

use crate::states::market::{Amm, OracleSource};

use crate::helpers::amm;
// use crate::helpers::casting::{cast, cast_to_i128, cast_to_i64, cast_to_u128};
// use crate::helpers::constants::*;

pub fn get_mark_price(a: &Amm) -> Result<u128, ContractError> {
    amm::calculate_price(
        a.quote_asset_reserve,
        a.base_asset_reserve,
        a.peg_multiplier,
    )

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
    price_oracle: &Addr,
    clock_slot: u64,
) -> Result<(i128, i128, u128, u128, i64), ContractError> {
    let (oracle_px, oracle_twap, oracle_conf, oracle_twac, oracle_delay) =
        match a.oracle_source {
            OracleSource::Oracle => get_oracle_data(a, &price_oracle, clock_slot)?,
            OracleSource::Simulated => (0, 0, 0, 0, 0),
            OracleSource::Zero => (0, 0, 0, 0, 0),
        };
    Ok((
        oracle_px,
        oracle_twap,
        oracle_conf,
        oracle_twac,
        oracle_delay,
    ))
}
