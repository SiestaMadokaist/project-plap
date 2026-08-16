use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceModelId(pub u32);
displayable!(InferenceModelId);
