use domain::storage::{StoragePath, StoragePrefix};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetListPayload {
    pub prefix: StoragePrefix,
}
json_type!(GetListPayload);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetListResponse {
    pub paths: Vec<StoragePath>,
}
json_type!(GetListResponse);
impl DTO for GetListResponse {}
