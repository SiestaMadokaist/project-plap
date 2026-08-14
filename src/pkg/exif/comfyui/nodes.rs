use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    #[serde(rename = "CLIPTextEncode")]
    ClipTextEncode,
    #[serde(rename = "CheckpointLoaderSimple")]
    CheckpointLoaderSimple,
    #[serde(other, rename = "other")]
    Other,
}
displayable!(NodeType);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComfyNode {
    #[serde(rename = "type")]
    pub tipe: NodeType,
    pub widgets_values: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComfyWorkflow {
    pub id: String,
    pub nodes: Vec<ComfyNode>,
}

impl ComfyNode {
    pub fn values(&self) -> Result<Vec<String>, serde_json::Error> {
        if self.tipe == NodeType::Other {
            return Ok(vec![]);
        }
        let nodes: Result<Vec<String>, serde_json::Error> =
            serde_json::from_value(self.widgets_values.clone());
        nodes
    }
}
