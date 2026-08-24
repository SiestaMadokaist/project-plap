use serde::{Deserialize, Serialize};

use crate::pkg::civitai::{
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
    #[serde(rename = "type")]
    tipe: ModelType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelVersionDTO {
    pub id: civitai::typing::VersionId,
    name: String,
    #[serde(rename = "modelId")]
    pub model_id: civitai::typing::ModelId,
    #[serde(rename = "baseModel")]
    base_model: BaseModel,
    model: ModelVersionModelDTO,
}

impl ModelVersionDTO {
    pub fn name(&self) -> String {
        let m = &self.model;
        let name = format!("{}-{}", m.name, self.name);
        name.to_lowercase().replace(" ", "_")
    }

    pub fn category(&self) -> typing::ModelCategory {
        if matches!(self.model.tipe, ModelType::Lora) {
            typing::ModelCategory::Loras
        } else {
            match self.base_model {
                BaseModel::Illustrious => typing::ModelCategory::Checkpoints,
                BaseModel::Krea2 => typing::ModelCategory::DiffusionModels,
                BaseModel::ZIT => typing::ModelCategory::DiffusionModels,
                BaseModel::Anima => typing::ModelCategory::DiffusionModels,
                BaseModel::Other => typing::ModelCategory::DiffusionModels,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::{displayable, pkg::civitai::dto::model_version::ModelVersionDTO};

    #[derive(Debug, thiserror::Error, Serialize)]
    enum E {
        Openfile(String),
        Serialize(String),
    }
    displayable!(E);

    #[test]
    fn shape_test_checkpoint() -> Result<(), E> {
        let buffer =
            std::fs::read("./samples/inputs/jsons/infras/civitai/resp.version.checkpoint.json")
                .map_err(|x| E::Openfile(x.to_string()))?;
        let resp: ModelVersionDTO =
            serde_json::from_slice(&buffer).map_err(|x| E::Serialize(x.to_string()))?;
        assert_eq!(resp.id.0, 3211163);
        assert_eq!(resp.model_id.0, 1998102);
        Ok(())
    }

    #[test]
    fn shape_test_lora() -> Result<(), E> {
        let buffer = std::fs::read("./samples/inputs/jsons/infras/civitai/resp.version.lora.json")
            .map_err(|x| E::Openfile(x.to_string()))?;
        let resp: ModelVersionDTO =
            serde_json::from_slice(&buffer).map_err(|x| E::Serialize(x.to_string()))?;
        assert_eq!(resp.id.0, 2116314);
        assert_eq!(resp.model_id.0, 880852);
        Ok(())
    }
}
