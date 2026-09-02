use domain::storage::{DirTree, StoragePrefix};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::response::DTO;
#[derive(Debug, Serialize, Deserialize)]
pub struct GetListPayload {
    pub prefix: StoragePrefix,
    pub recursive: bool,
}
json_type!(GetListPayload);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetListResponse(pub DirTree);
json_type!(GetListResponse);
impl DTO for GetListResponse {}
