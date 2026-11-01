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
    /** @todo: move default implementation to backend specific crates and use uuid.v4() */
    fn default() -> Self {
        Self("0000-0000-0000-0000".into())
    }
}
