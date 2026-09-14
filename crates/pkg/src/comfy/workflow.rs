use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    comfy::nodes::{
        KnownNode::{self, ClipText, SD3Latent, UnetLoader},
        PromptNode,
    },
    types::unit::Px,
};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PostRequests {
    client_id: String,
    prompt: HashMap<String, PromptNode>,
    extra_data: serde_json::Value,
}

impl PostRequests {
    pub fn set_positive(&mut self, text: &str) {
        let Some(PromptNode::Known(ClipText(node))) = self
            .prompt
            .values_mut()
            .find(|v| matches!(v, PromptNode::Known(ClipText(_))))
        else {
            return;
        };
        node.inputs.text = text.into();
    }

    pub fn set_resolution(&mut self, width: Px, height: Px) {
        let Some(PromptNode::Known(SD3Latent(node))) = self
            .prompt
            .values_mut()
            .find(|v| matches!(v, PromptNode::Known(SD3Latent(_))))
        else {
            return;
        };
        node.inputs.width = width;
        node.inputs.height = height;
    }
    pub fn set_checkpoint(&mut self, v: &str) {
        let Some(PromptNode::Known(UnetLoader(node))) = self
            .prompt
            .values_mut()
            .find(|v| matches!(v, PromptNode::Known(UnetLoader(_))))
        else {
            return;
        };
        node.inputs.unet_name = v.into();
    }
}

#[cfg(test)]
mod tests {
    use crate::{comfy::workflow::PostRequests, utils::testhelper};

    #[test]
    fn shape_test() -> Result<(), testhelper::Error> {
        let data: PostRequests = testhelper::read_json(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/infras/comfyui/post-request.json"
        ))?;
        assert_eq!(data.prompt.len(), 10);
        Ok(())
    }
}
