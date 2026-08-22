use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

// as far as civitai is concerned this is just "VersionId"; application/ports
// only ever sees it as the provider-agnostic pkg::id::InferenceModelId.
pub type VersionId = crate::pkg::id::InferenceModelId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelId(pub u32);
displayable!(ModelId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaseModel {
    #[serde(rename = "Illustrious")]
    Illustrious,
    #[serde(rename = "Krea 2")]
    Krea2,
    #[serde(rename = "ZImageTurbo")]
    ZIT,
    #[serde(rename = "Anima")]
    Anima,
    #[serde(other)]
    Other,
}

// probably should be refactored into comfyui pkg
// this is actually comfyui concern, not civitai concern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCategory {
    // vae and stuff is baked in
    #[serde(rename = "checkpoints")]
    Checkpoints,
    // vae and stuff is baked in
    #[serde(rename = "diffusion_models")]
    DiffusionModels,
    #[serde(rename = "loras")]
    Loras,
}

displayable!(ModelCategory);
