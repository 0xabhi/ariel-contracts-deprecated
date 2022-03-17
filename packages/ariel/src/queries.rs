use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    
    MarketInitialized {
        market_index: u64
    },
    ValidOracleForMarket {
        oracle: Addr,
        market_index: u64
    },
    
}

