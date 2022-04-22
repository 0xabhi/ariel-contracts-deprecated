use crate::ContractError;
use crate::helpers::constants::{MAXIMUM_MARGIN_RATIO, MINIMUM_MARGIN_RATIO};

pub fn validate_margin(
    margin_ratio_initial: u32,
    margin_ratio_partial: u32,
    margin_ratio_maintenance: u32,
) -> Result<bool, ContractError> {
    if !(MINIMUM_MARGIN_RATIO.u128()..=MAXIMUM_MARGIN_RATIO.u128()).contains(&(margin_ratio_initial as u128)) {
        return Err(ContractError::InvalidMarginRatio);
    }

    if margin_ratio_initial < margin_ratio_partial {
        return Err(ContractError::InvalidMarginRatio);
    }

    if !(MINIMUM_MARGIN_RATIO.u128()..=MAXIMUM_MARGIN_RATIO.u128()).contains(&(margin_ratio_partial as u128)) {
        return Err(ContractError::InvalidMarginRatio);
    }

    if margin_ratio_partial < margin_ratio_maintenance {
        return Err(ContractError::InvalidMarginRatio);
    }

    if !(MINIMUM_MARGIN_RATIO.u128()..=MAXIMUM_MARGIN_RATIO.u128()).contains(&(margin_ratio_maintenance as u128)) {
        return Err(ContractError::InvalidMarginRatio);
    }

    Ok(true)
}
