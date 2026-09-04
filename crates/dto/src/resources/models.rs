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

/// Preview request for one collapsed entry. Any part may be empty/absent; at most
/// a handful of image samples are requested at once.
#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewPayload {
    #[serde(default)]
    pub images: Vec<StoragePath>,
    pub json: Option<StoragePath>,
}
json_type!(PreviewPayload);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreviewResponse {
    /// presigned GET urls for the requested image samples, in request order
    #[serde(default)]
    pub image_urls: Vec<String>,
    /// raw text of the json sibling, if one was requested
    pub json: Option<String>,
}
json_type!(PreviewResponse);
impl DTO for PreviewResponse {}
