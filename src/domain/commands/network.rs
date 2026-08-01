use serde::{Deserialize, Serialize};

use crate::pkg::types::id::ModelProvider;

#[derive(Serialize, Deserialize)]
pub struct NetworkArgs {
    provider: ModelProvider,
    /**
     * e.g: s3://<bucket>/path
     * modelId
     * https://something/endpoint
     */
    path: String,
}
