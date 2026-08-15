use serde::{Deserialize, Serialize};

use crate::infras::civitai::{self, typing::BaseModel};

/// more details on ../samples/resp.model.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelDetailDTO {
    id: civitai::typing::ModelId,
    name: String,
    #[serde(rename = "baseModels")]
    base_models: Vec<BaseModel>,
}
