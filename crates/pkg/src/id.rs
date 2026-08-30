use serde::{Deserialize, Serialize};

use crate::macros::displayable;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct InferenceModelId(pub u32);
displayable!(InferenceModelId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId(pub String);
displayable!(TraceId);
