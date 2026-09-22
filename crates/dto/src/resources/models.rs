use domain::storage::{DirTree, StorageBucket, StoragePath, StoragePrefix};
use pkg::{json_type, types::time::Second};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewTarget {
    pub path: StoragePath,
    pub ttl: Second,
}

/// Preview request for one collapsed entry. Any part may be empty/absent; at most
/// a handful of image samples are requested at once.
#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewPayload {
    #[serde(default)]
    pub images: Vec<PreviewTarget>,
    pub json: Option<PreviewTarget>,
    /// the model file itself — presigned so the modal can offer a direct download
    pub safetensors: Option<PreviewTarget>,
}
json_type!(PreviewPayload);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreviewResponse {
    /// presigned GET urls for the requested image samples, in request order
    #[serde(default)]
    pub image_urls: Vec<String>,
    /// raw text of the json sibling, if one was requested
    pub json: Option<String>,
    /// presigned GET url for the model file, if one was requested
    pub safetensors_url: Option<String>,
}
json_type!(PreviewResponse);
impl DTO for PreviewResponse {}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct DeletePayload {
    pub paths: Vec<StoragePath>,
}
json_type!(DeletePayload);

#[derive(Debug, Serialize, Clone, Deserialize, Default)]
pub struct DeleteResponse(());
impl DTO for DeleteResponse {}
