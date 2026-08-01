use serde::{Deserialize, Serialize};

use crate::{
    domain::commands::{compute::ComputeArgs, inference::InferenceArgs, network::NetworkArgs},
    pkg::types::time::Timestamp,
};

#[derive(Serialize, Deserialize)]
pub struct CommandDomain {
    pub action_id: String,
    pub priority: u64,
    pub status: String,
    pub created_at: Timestamp,
    pub ttl: Timestamp,
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "data", rename_all = "lowercase")]
pub enum Action {
    Inference(InferenceArgs),
    Network(NetworkArgs),
    Compute(ComputeArgs),
}
