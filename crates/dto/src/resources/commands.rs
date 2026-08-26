use domain::commands::command::{CommandDomain, CommandStage};
use pkg::json_type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetListPayload {
    pub stage: CommandStage,
    pub limit: i32,
}
json_type!(GetListPayload);

#[derive(Debug, Serialize, Deserialize)]
pub struct GetListResponse {
    pub commands: Vec<CommandDomain>,
}
json_type!(GetListResponse);
