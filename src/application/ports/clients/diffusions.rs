use crate::domain::commands::inference::InferenceConfig;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait DiffusionClient {
    async fn generate(&self, params: &InferenceConfig<String>) -> anyhow::Result<()>;
}
