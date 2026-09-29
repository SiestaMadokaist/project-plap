use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    comfy::workflow::{
        api::nodes::{
            ApiNode,
            Known::{ClipText, SD3Latent, UnetLoader},
        },
        info::ExtraInfo,
    },
    types::unit::Px,
};

pub mod inputs;
pub mod nodes;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ExportApi {
    client_id: String,
    prompt: HashMap<String, ApiNode>,
    extra_data: ExtraInfo,
}

impl ExportApi {
    pub fn set_positive(&mut self, text: &str) {
        let Some(ApiNode::Known(ClipText(node))) = self
            .prompt
            .values_mut()
            .find(|v| matches!(v, ApiNode::Known(ClipText(_))))
        else {
            return;
        };
        node.inputs.text = text.into();
    }

    pub fn set_resolution(&mut self, width: Px, height: Px) {
        let Some(ApiNode::Known(SD3Latent(node))) = self
            .prompt
            .values_mut()
            .find(|v| matches!(v, ApiNode::Known(SD3Latent(_))))
        else {
            return;
        };
        node.inputs.width = width;
        node.inputs.height = height;
    }
    pub fn set_checkpoint(&mut self, v: &str) {
        let Some(ApiNode::Known(UnetLoader(node))) = self
            .prompt
            .values_mut()
            .find(|v| matches!(v, ApiNode::Known(UnetLoader(_))))
        else {
            return;
        };
        node.inputs.unet_name = v.into();
    }
}

#[cfg(test)]
mod tests {
    use crate::{comfy::workflow::api::ExportApi, utils::testhelper};

    #[test]
    fn shape_test() -> Result<(), testhelper::Error> {
        let data: ExportApi = testhelper::read_json(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/infras/comfyui/workflow-api.json"
        ))?;
        assert_eq!(data.prompt.len(), 10);
        Ok(())
    }
}
