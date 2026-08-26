use domain::commands::inference::InferenceConfig;
use pkg::exif::comfyui::nodes::ComfyWorkflow;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait DiffusionClient {
    async fn generate(
        &self,
        params: &InferenceConfig<String>,
        workflow: Option<ComfyWorkflow>,
    ) -> anyhow::Result<()>;
}
