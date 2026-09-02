use domain::storage::{StoragePath, StoragePrefix};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::{resources::list::ListResponse, response::DTO};
#[derive(Debug, Serialize, Deserialize)]
pub struct GetListPayload {
    pub prefix: StoragePrefix,
}
json_type!(GetListPayload);

pub type GetListResponse = ListResponse<StoragePath>;
json_type!(GetListResponse);
impl DTO for GetListResponse {}
