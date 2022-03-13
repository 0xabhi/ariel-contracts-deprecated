use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Clone, Debug, JsonSchema, Copy, Serialize, Deserialize, PartialEq)]
pub enum PositionDirection {
    Long,
    Short,
}

impl Default for PositionDirection {
    // UpOnly
    fn default() -> Self {
        PositionDirection::Long
    }
}

#[derive(Clone, Debug, JsonSchema, Copy, Serialize, Deserialize, PartialEq)]
pub enum SwapDirection {
    Add,
    Remove,
}
