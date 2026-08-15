use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionId(pub u32);
displayable!(VersionId);

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
