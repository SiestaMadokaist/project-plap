use serde::{Deserialize, Serialize};

use crate::comfy::workflow;

pub mod nodes;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowId(String);

impl WorkflowId {
    pub fn to_string(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    id: WorkflowId,
    revision: u32,
    last_node_id: u32,
    last_link_id: u32,
    nodes: Vec<workflow::info::nodes::ExifNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraInfo {
    pub workflow: Workflow,
}

impl ExtraInfo {
    pub fn nodes(&self) -> &Vec<workflow::info::nodes::ExifNode> {
        &self.workflow.nodes
    }
}
#[cfg(test)]
mod tests {
    use crate::comfy::workflow::info::{
        nodes::{
            ExifNode::{self},
            Known,
        },
        ExtraInfo,
    };

    #[test]
    fn shape_test() {
        let buffer = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/infras/comfyui/pnginfo.json"
        ))
        .expect("must succeed");
        let pnginfo: ExtraInfo = serde_json::from_slice(&buffer).expect("must success parse");
        assert_eq!(
            pnginfo.workflow.id.to_string(),
            "e334c90f-7c39-4b78-80fd-6cd040e30265"
        );
        let nodes = pnginfo.nodes();
        let mut mprompt: Option<&Known> = None;
        {
            for node in nodes {
                if matches!(node, ExifNode::Unknown(_)) {
                    continue;
                }
                if let ExifNode::Known(b) = node {
                    let k = b.as_ref();
                    if let Known::ClipTextEncode(_) = k {
                        mprompt = Some(k)
                    }
                }
            }
        }
        if let Known::ClipTextEncode(nv) = mprompt.expect("exists") {
            let wv = nv.widgets_values_named.as_ref().unwrap();
            assert!(wv.text.starts_with("(( highly detailed, best quality"));
            assert!(wv.text.ends_with("and showing seductive smirk.\n"));
            assert!(wv.text.contains("Ganyu Costume with light blue wigs"));
        } else {
            assert_eq!("TEXT must exist", "text doesnt exist");
        }
    }
}
