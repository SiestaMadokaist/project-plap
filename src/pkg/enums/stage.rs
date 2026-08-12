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
