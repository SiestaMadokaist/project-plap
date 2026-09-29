use serde::{Deserialize, Serialize};

use crate::types::unit::Px;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputSlot {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub link: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputSlot {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub links: Option<Vec<i64>>,
}

/// Canvas-graph node, as embedded under `extra_data.extra_pnginfo.workflow.nodes`.
/// `W` is the node-specific shape of `widgets_values_named`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node<W> {
    pub id: u32,
    #[serde(rename = "type")]
    pub r#type: String,
    pub pos: (f64, f64),
    pub inputs: Vec<InputSlot>,
    pub outputs: Vec<OutputSlot>,
    size: (f64, f64),
    flags: serde_json::Value,
    order: u32,
    mode: u8,
    properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    widgets_values: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widgets_values_named: Option<W>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bgcolor: Option<String>,
    #[serde(rename = "showAdvanced", skip_serializing_if = "Option::is_none")]
    show_advanced: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteWidgets {
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaeLoaderWidgets {
    vae_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KSamplerWidgets {
    seed: i64,
    control_after_generate: String,
    steps: u32,
    cfg: f64,
    sampler_name: String,
    scheduler: String,
    denoise: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageSaverSimpleWidgets {
    filename: String,
    path: String,
    extension: String,
    lossless_webp: bool,
    quality_jpeg_or_webp: u32,
    optimize_png: bool,
    embed_workflow: bool,
    save_workflow_as_json: bool,
    counter: u32,
    time_format: String,
    show_preview: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSamplingAuraFlowWidgets {
    shift: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipLoaderWidgets {
    clip_name: String,
    #[serde(rename = "type")]
    kind: String,
    device: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptySD3LatentImageWidgets {
    width: Px,
    height: Px,
    batch_size: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnetLoaderWidgets {
    unet_name: String,
    weight_dtype: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipTextEncodeWidgets {
    pub text: String,
}

pub type VAEDecodeNode = Node<serde_json::Value>;
pub type NoteNode = Node<NoteWidgets>;
pub type VAELoaderNode = Node<VaeLoaderWidgets>;
pub type KSamplerNode = Node<KSamplerWidgets>;
pub type ImageSaverSimpleNode = Node<ImageSaverSimpleWidgets>;
pub type ModelSamplingAuraFlowNode = Node<ModelSamplingAuraFlowWidgets>;
pub type ConditioningZeroOutNode = Node<serde_json::Value>;
pub type ClipLoaderNode = Node<ClipLoaderWidgets>;
pub type EmptySD3LatentImageNode = Node<EmptySD3LatentImageWidgets>;
pub type UnetLoaderNode = Node<UnetLoaderWidgets>;
pub type ClipTextEncodeNode = Node<ClipTextEncodeWidgets>;

/// Each variant's node struct carries its own public `r#type` field (see
/// [`Node`]), so the discriminant can't be an internally-tagged `type` key
/// (serde would consume it before the variant ever saw it). Serialization
/// stays derived (`untagged` just writes whichever variant is active);
/// deserialization is hand-rolled below to peek at `type` without eating it.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Known {
    VAEDecode(VAEDecodeNode),
    Note(NoteNode),
    VAELoader(VAELoaderNode),
    KSampler(KSamplerNode),
    ImageSaverSimple(ImageSaverSimpleNode),
    ModelSamplingAuraFlow(ModelSamplingAuraFlowNode),
    ConditioningZeroOut(ConditioningZeroOutNode),
    ClipLoader(ClipLoaderNode),
    EmptySD3LatentImage(EmptySD3LatentImageNode),
    UnetLoader(UnetLoaderNode),
    ClipTextEncode(ClipTextEncodeNode),
}

impl<'de> Deserialize<'de> for Known {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let kind = value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("type"))?;

        macro_rules! variant {
            ($ident:ident) => {
                serde_json::from_value(value.clone())
                    .map(Known::$ident)
                    .map_err(serde::de::Error::custom)
            };
        }

        match kind {
            "VAEDecode" => variant!(VAEDecode),
            "Note" => variant!(Note),
            "VAELoader" => variant!(VAELoader),
            "KSampler" => variant!(KSampler),
            "Image Saver Simple" => variant!(ImageSaverSimple),
            "ModelSamplingAuraFlow" => variant!(ModelSamplingAuraFlow),
            "ConditioningZeroOut" => variant!(ConditioningZeroOut),
            "CLIPLoader" => variant!(ClipLoader),
            "EmptySD3LatentImage" => variant!(EmptySD3LatentImage),
            "UNETLoader" => variant!(UnetLoader),
            "CLIPTextEncode" => variant!(ClipTextEncode),
            other => Err(serde::de::Error::custom(format!(
                "unknown comfy node type: {other}"
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ExifNode {
    Known(Box<Known>),
    Unknown(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn invariant_known_node_type() {
        let original = json!({
            "id": 9,
            "type": "VAEDecode",
            "pos": [1266.6474256003016, -77.27004541921318],
            "size": [225.0, 54.0],
            "flags": {},
            "order": 9,
            "mode": 0,
            "inputs": [
                { "name": "samples", "type": "LATENT", "link": 10 },
                { "name": "vae", "type": "VAE", "link": 30 }
            ],
            "outputs": [
                { "name": "IMAGE", "type": "IMAGE", "links": [33] }
            ],
            "properties": {
                "Node name for S&R": "VAEDecode",
                "cnr_id": "comfy-core",
                "ver": "0.31.0"
            }
        });

        let node: ExifNode = serde_json::from_value(original.clone()).unwrap();
        assert!(matches!(node, ExifNode::Known(ref b) if matches!(**b, Known::VAEDecode(_))));

        let written = serde_json::to_value(&node).unwrap();
        assert_eq!(written, original);
    }

    #[test]
    fn invariant_unknown_node_type() {
        let original = json!({
            "id": 99,
            "type": "SomeCustomNode",
            "pos": [0.0, 0.0],
            "size": [100.0, 100.0],
            "flags": {},
            "order": 0,
            "mode": 0,
            "inputs": [],
            "outputs": [],
            "properties": {}
        });

        let node: ExifNode = serde_json::from_value(original.clone()).unwrap();
        assert!(matches!(node, ExifNode::Unknown(_)));

        let written = serde_json::to_value(&node).unwrap();
        assert_eq!(written, original);
    }

    #[test]
    fn shape_test() -> Result<(), crate::utils::testhelper::Error> {
        let data: serde_json::Value = crate::utils::testhelper::read_json(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/infras/comfyui/workflow-api.json"
        ))?;
        let raw_nodes = &data["extra_data"]["extra_pnginfo"]["workflow"]["nodes"];
        let nodes: Vec<ExifNode> = serde_json::from_value(raw_nodes.clone()).unwrap();

        assert_eq!(nodes.len(), 11);
        assert!(
            nodes.iter().all(|n| matches!(n, ExifNode::Known(_))),
            "expected every sample node to be a Known variant"
        );
        Ok(())
    }
}
