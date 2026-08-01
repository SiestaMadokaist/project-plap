use crate::domain::{commands::inference::InferenceArgs, errors::DomainError};

#[allow(async_fn_in_trait)]
pub trait DiffusionClient {
    async fn generate(&self, params: InferenceArgs) -> anyhow::Result<()>;
}
