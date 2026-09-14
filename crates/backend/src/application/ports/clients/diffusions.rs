use domain::{commands::inference::InferenceConfig, errors::DomainError};
use pkg::comfy;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait DiffusionClient {
    async fn generate(
        &self,
        params: &InferenceConfig<String>,
        workflow: Option<comfy::workflow::PostRequests>,
    ) -> Result<(), DomainError>;
}
