use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetUser {
        user_address: String
    },
    GetUserMarketPosition {
        user_address: String,
        index: u64
    },
    GetAdmin {},
    IsExchangePaused {},
    IsFundingPaused {},
    AdminControlsPrices{},
    GetVaults{},
    GetMarginRatio{},
    GetPartialLiquidationClosePercentage{},
    GetPartialLiquidationPenaltyPercentage{},
    GetFullLiquidationPenaltyPercentage{},
    GetPartialLiquidatorSharePercentage{},
    GetFullLiquidatorSharePercentage{},
    GetMaxDepositLimit{},
    GetFeeStructure{},
    //TODO::get user market positions which returns array
    // TODO:: get all the history with bound length like 1-100, 101-200 etc.String
    GetCurveHistoryLength{},
    GetCurveHistory{
        index: u64
    },
    GetDepositHistoryLength{},
    GetDepositHistory{
        user_address: String,
        index: u64
    },
    GetFundingPaymentHistoryLength{},
    GetFundingPaymentHistory{
        user_address: String,
        index: u64
    },
    GetFundingRateHistoryLength{},
    GetFundingRateHistory{
        index: u64
    },

    GetLiquidationHistoryLength{},
    GetLiquidationHistory{
        user_address: String,
        index: u64
    },
    GetTradeHistoryLength{},
    GetTradeHistory{
        user_address: String,
        index: u64
    },
    GetMarketInfo{
        market_index: u64
    }
}

