use crate::domain::commands::inference::InferenceConfig;

#[async_trait::async_trait(?Send)]
pub trait DiffusionClient {
    async fn generate(&self, params: &InferenceConfig<String>) -> anyhow::Result<()>;
}
