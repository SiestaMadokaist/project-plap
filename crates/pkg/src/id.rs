use serde::{Deserialize, Serialize};

use crate::macros::displayable;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct InferenceModelId(pub u32);
displayable!(InferenceModelId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId(String);
displayable!(TraceId);

impl TraceId {
    pub fn new(s: &str) -> Self {
        Self(s.into())
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}
