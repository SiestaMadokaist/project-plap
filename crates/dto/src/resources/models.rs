use domain::storage::{DirTree, StorageBucket, StoragePath, StoragePrefix};
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

/// Preview request for one collapsed entry. Either sibling may be absent.
#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewPayload {
    pub image: Option<StoragePath>,
    pub json: Option<StoragePath>,
}
json_type!(PreviewPayload);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreviewResponse {
    /// presigned GET url for the image sibling, if one was requested
    pub image_url: Option<String>,
    /// raw text of the json sibling, if one was requested
    pub json: Option<String>,
}
json_type!(PreviewResponse);
impl DTO for PreviewResponse {}
