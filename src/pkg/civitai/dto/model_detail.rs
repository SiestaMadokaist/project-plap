use serde::{Deserialize, Serialize};

use crate::pkg::civitai::{self, typing::BaseModel};

/// more details on ../samples/resp.model.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelDetailDTO {
    id: civitai::typing::ModelId,
    name: String,
    #[serde(rename = "baseModels")]
    base_models: Vec<BaseModel>,
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::{
        displayable,
        pkg::civitai::dto::model_detail::ModelDetailDTO,
    };

    #[derive(Debug, thiserror::Error, Serialize)]
    enum E {
        Openfile(String),
        Serialize(String),
    }
    displayable!(E);

    #[test]
    fn shape_test() -> Result<(), E> {
        let buffer = std::fs::read("./samples/inputs/jsons/infras/civitai/resp.model.json")
            .map_err(|x| E::Openfile(x.to_string()))?;
        let resp: ModelDetailDTO =
            serde_json::from_slice(&buffer).map_err(|x| E::Serialize(x.to_string()))?;
        assert_eq!(resp.id.0, 1998102);
        assert_eq!(resp.name, "YozakuraKiss");
        Ok(())
    }
}
