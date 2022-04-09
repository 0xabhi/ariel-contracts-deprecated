use crate::ContractError;
use crate::helpers::constants::{MAXIMUM_MARGIN_RATIO, MINIMUM_MARGIN_RATIO};

pub fn validate_margin(
    margin_ratio_initial: u32,
    margin_ratio_partial: u32,
    margin_ratio_maintenance: u32,
) -> Result<bool, ContractError> {
    if !(MINIMUM_MARGIN_RATIO..=MAXIMUM_MARGIN_RATIO).contains(&margin_ratio_initial) {
        return Err(ContractError::InvalidMarginRatio);
    }

    if margin_ratio_initial < margin_ratio_partial {
        return Err(ContractError::InvalidMarginRatio);
    }

    if !(MINIMUM_MARGIN_RATIO..=MAXIMUM_MARGIN_RATIO).contains(&margin_ratio_partial) {
        return Err(ContractError::InvalidMarginRatio);
    }

    if margin_ratio_partial < margin_ratio_maintenance {
        return Err(ContractError::InvalidMarginRatio);
    }

    if !(MINIMUM_MARGIN_RATIO..=MAXIMUM_MARGIN_RATIO).contains(&margin_ratio_maintenance) {
        return Err(ContractError::InvalidMarginRatio);
    }

    Ok(true)
}
