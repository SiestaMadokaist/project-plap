use domain::storage::{DirTree, StorageBucket, StoragePrefix};
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
pub struct GetListResponse {
    /// Bucket the listed keys live in — the frontend needs it to build a cp command.
    pub bucket: StorageBucket,
    pub tree: DirTree,
}
json_type!(GetListResponse);
impl DTO for GetListResponse {}
