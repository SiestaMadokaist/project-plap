use serde::{Deserialize, Serialize};

use crate::macros::displayable;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct InferenceModelId(pub u32);
displayable!(InferenceModelId);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TraceFlag {
    RequestHeader,
    ServerGenerated,
}

impl TraceFlag {
    pub fn flag(&self) -> char {
        match &self {
            TraceFlag::RequestHeader => 'f',
            TraceFlag::ServerGenerated => 'b',
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId {
    flag: TraceFlag,
    uid: uuid::Uuid,
}

impl TraceId {
    pub fn new(flag: TraceFlag, uid: uuid::Uuid) -> Self {
        Self { flag, uid }
    }

    pub fn uuid(&self) -> &uuid::Uuid {
        &self.uid
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.flag.flag(), self.uid)
    }
}
