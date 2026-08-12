use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::{commands::command::ModelProvider, storage::StoragePath};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkAction {
    Download,
    Upload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkArgs {
    pub provider: ModelProvider,
    pub action: NetworkAction,
    /**
     * e.g: s3://<bucket>/path
     * modelId
     * https://something/endpoint
     */
    pub remote: StoragePath,
    // relative path since workdir
    pub local: PathBuf,
}
