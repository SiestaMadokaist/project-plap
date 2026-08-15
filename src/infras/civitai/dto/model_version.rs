use serde::{Deserialize, Serialize};

use crate::infras::civitai::{
    self,
    typing::{self, BaseModel},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
enum ModelType {
    #[serde(rename = "Checkpoint")]
    Checkpoint,
    #[serde(rename = "LORA")]
    Lora,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModelVersionModelDTO {
    name: String,
    tipe: ModelType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelVersionDTO {
    pub id: civitai::typing::VersionId,
    pub name: String,
    #[serde(rename = "modelId")]
    pub model_id: civitai::typing::ModelId,
    #[serde(rename = "baseModel")]
    base_model: BaseModel,
    model: ModelVersionModelDTO,
}

impl ModelVersionDTO {
    pub fn category(&self) -> typing::ModelCategory {
        if matches!(self.model.tipe, ModelType::Lora) {
            typing::ModelCategory::Loras
        } else {
            match self.base_model {
                BaseModel::Illustrious => typing::ModelCategory::Checkpoints,
                BaseModel::Krea2 => typing::ModelCategory::DiffusionModels,
                BaseModel::ZIT => typing::ModelCategory::DiffusionModels,
                BaseModel::Other => typing::ModelCategory::DiffusionModels,
            }
        }
    }
}
