use serde::{Deserialize, Serialize};

use crate::comfy::workflow::api::inputs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeInput<I> {
    pub inputs: I,
    #[serde(rename = "_meta")]
    pub meta: Meta,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "class_type")]
pub enum Known {
    #[serde(rename = "UNETLoader")]
    UnetLoader(NodeInput<inputs::UnetLoader>),
    #[serde(rename = "CLIPLoader")]
    ClipLoader(NodeInput<inputs::ClipLoader>),
    #[serde(rename = "CLIPTextEncode")]
    ClipText(NodeInput<inputs::ClipTextEncode>),
    #[serde(rename = "EmptySD3LatentImage")]
    SD3Latent(NodeInput<inputs::EmptySD3LatentImage>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ApiNode {
    Known(Known),
    Unknown(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn invariant_known_class_type() {
        let original = json!({
            "class_type": "UNETLoader",
            "inputs": {
                "unet_name": "beret_mix_anima_real2.8d-v1.0.safetensors",
                "weight_dtype": "default"
            },
            "_meta": { "title": "Load Diffusion Model" }
        });

        let node: ApiNode = serde_json::from_value(original.clone()).unwrap();
        assert!(matches!(node, ApiNode::Known(Known::UnetLoader(_))));

        let written = serde_json::to_value(&node).unwrap();
        assert_eq!(written, original);
    }

    #[test]
    fn invariant_unknown_class_type() {
        let original = json!({
            "inputs": { "vae_name": "qwen_image_vae.safetensors" },
            "class_type": "VAELoader",
            "_meta": { "title": "Load VAE" }
        });

        let node: ApiNode = serde_json::from_value(original.clone()).unwrap();
        assert!(matches!(node, ApiNode::Unknown(_)));

        let written = serde_json::to_value(&node).unwrap();
        assert_eq!(written, original);
    }
}
