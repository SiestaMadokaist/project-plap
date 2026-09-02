use domain::commands::{
    command::{ActionId, CommandDomain, CommandStage},
    network::NetworkArgs,
};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetListPayload {
    pub stage: CommandStage,
    pub limit: i32,
}
json_type!(GetListPayload);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetListResponse {
    pub commands: Vec<CommandDomain>,
}
json_type!(GetListResponse);
impl DTO for GetListResponse {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePayload {
    pub action_id: ActionId,
}
json_type!(DeletePayload);

/// Body of `POST /agents/command/cp`: queue an s3 -> localhost model copy.
#[derive(Debug, Serialize, Deserialize)]
pub struct CpModelPayload {
    pub action_id: ActionId,
    pub args: NetworkArgs,
    pub priority: u64,
}
json_type!(CpModelPayload);

/// Response of `POST /agents/command/cp` — the queued command as persisted.
/// Backend and frontend share this shape, so a change on either side breaks the
/// other at compile time.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpModelResponse {
    pub command: CommandDomain,
}
json_type!(CpModelResponse);
impl DTO for CpModelResponse {}
