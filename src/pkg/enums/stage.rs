use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Development,
    Staging,
    Production,
}
displayable!(Stage);

impl From<&str> for Stage {
    fn from(value: &str) -> Self {
        match value {
            "development" => Stage::Development,
            "staging" => Stage::Staging,
            "production" => Stage::Production,
            _ => Stage::Development,
        }
    }
}
