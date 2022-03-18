use std::convert::TryInto;

use crate::error::ContractError;

pub fn cast<T: TryInto<U>, U>(t: T) -> Result<U, ContractError> {
    t.try_into().map_err(|_| ContractError::CastingFailure)
}

pub fn cast_to_i128<T: TryInto<i128>>(t: T) -> Result<i128, ContractError> {
    cast(t)
}

pub fn cast_to_u128<T: TryInto<u128>>(t: T) -> Result<u128, ContractError> {
    cast(t)
}

pub fn cast_to_i64<T: TryInto<i64>>(t: T) -> Result<i64, ContractError> {
    cast(t)
}
