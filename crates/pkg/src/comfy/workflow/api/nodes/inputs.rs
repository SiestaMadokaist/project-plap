use serde::{Deserialize, Serialize};

use crate::types::unit::Px;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnetLoader {
    pub unet_name: String,
    weight_dtype: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipLoader {
    clip_name: String,
    #[serde(rename = "type")]
    tipe: String,
    device: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Clip(String, i32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipTextEncode {
    pub text: String,
    clip: Clip,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptySD3LatentImage {
    pub width: Px,
    pub height: Px,
    pub batch_size: u8,
}
