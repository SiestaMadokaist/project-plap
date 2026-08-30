use std::rc::Rc;

use domain::commands::command::{ActionId, CommandDomain, CommandStage};
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
    pub commands: Rc<Vec<CommandDomain>>,
}
json_type!(GetListResponse);
impl DTO for GetListResponse {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePayload {
    pub action_id: ActionId,
}
