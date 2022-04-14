use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Map;

use ariel::types::{OracleSource, OracleStatus, OraclePriceData};

use crate::error::ContractError;

use crate::helpers::amm;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Market {
    pub market_name: String,
    pub initialized: bool,
    pub base_asset_amount_long: i128,
    pub base_asset_amount_short: i128,
    pub base_asset_amount: i128, // net market bias
    pub open_interest: u128,     // number of users in a position
    pub amm: Amm,
    pub margin_ratio_initial: u32,
    pub margin_ratio_partial: u32,
    pub margin_ratio_maintenance: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Amm {
    pub oracle: Addr,
    pub oracle_source: OracleSource,
    pub base_asset_reserve: u128,
    pub quote_asset_reserve: u128,
    pub cumulative_repeg_rebate_long: u128,
    pub cumulative_repeg_rebate_short: u128,
    pub cumulative_funding_rate_long: i128,
    pub cumulative_funding_rate_short: i128,
    pub last_funding_rate: i128,
    pub last_funding_rate_ts: u64,
    pub funding_period: u64,
    pub last_oracle_price_twap: i128,
    pub last_mark_price_twap: u128,
    pub last_mark_price_twap_ts: u64,
    pub sqrt_k: u128,
    pub peg_multiplier: u128,
    pub total_fee: u128,
    pub total_fee_minus_distributions: u128,
    pub total_fee_withdrawn: u128,
    pub minimum_quote_asset_trade_size: u128,
    pub last_oracle_price_twap_ts: u64,
    pub last_oracle_price: i128,
    pub minimum_base_asset_trade_size: u128,
}

pub const MARKETS: Map<u64, Market> = Map::new("markets");

impl Amm {
    pub fn mark_price(&self) -> Result<u128, ContractError> {
        amm::calculate_price(
            self.quote_asset_reserve,
            self.base_asset_reserve,
            self.peg_multiplier,
        )
    }

    pub fn get_oracle_price(
        &self,
        price_oracle: &Addr,
        now: u64,
    ) -> Result<OraclePriceData, ContractError> {
        // match self.oracle_source {
        //     OracleSource::Pyth => self.get_pyth_price(price_oracle, clock_slot),
        //     OracleSource::Switchboard => self.get_switchboard_price(price_oracle, clock_slot),
        // }
        Ok(OraclePriceData {
            price : 0,
            confidence : 0,
            delay : 0,
            has_sufficient_number_of_data_points : false,
        })
    }

    pub fn get_oracle_twap(&self, price_oracle: &Addr) -> Result<Option<i128>, ContractError> {
        // match self.oracle_source {
            // OracleSource::Pyth => Ok(Some(self.get_pyth_twap(price_oracle)?)),
            // OracleSource::Switchboard => Ok(None),
        // }
        Ok(Some(0))
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum LiquidationType {
    NONE,
    PARTIAL,
    FULL,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationStatus {
    pub liquidation_type: LiquidationType,
    pub margin_requirement: u128,
    pub total_collateral: u128,
    pub unrealized_pnl: i128,
    pub adjusted_total_collateral: u128,
    pub base_asset_value: u128,
    pub margin_ratio: u128,
    pub market_statuses: Vec<MarketStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketStatus {
    pub market_index: u64,
    pub partial_margin_requirement: u128,
    pub maintenance_margin_requirement: u128,
    pub base_asset_value: u128,
    pub mark_price_before: u128,
    pub close_position_slippage: Option<i128>,
    pub oracle_status: OracleStatus,
}

