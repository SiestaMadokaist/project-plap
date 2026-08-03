use serde::{Deserialize, Serialize};

use crate::domain::commands::command::ModelProvider;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkArgs {
    provider: ModelProvider,
    /**
     * e.g: s3://<bucket>/path
     * modelId
     * https://something/endpoint
     */
    path: String,
}
