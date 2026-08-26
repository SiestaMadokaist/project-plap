use crate::application::ports::clients::diffusions::DiffusionClient;
use domain::commands::inference::InferenceConfig;
use pkg::exif::comfyui::nodes::ComfyWorkflow;

pub struct A1111 {
    base_url: String,
}

impl A1111 {
    pub fn new(base_url: String) -> Self {
        A1111 { base_url }
    }
}

#[async_trait::async_trait(?Send)]
impl DiffusionClient for A1111 {
    async fn generate(
        &self,
        _params: &InferenceConfig<String>,
        _workflow: Option<ComfyWorkflow>,
    ) -> anyhow::Result<()> {
        let _ = &self.base_url;
        todo!()
    }
}
