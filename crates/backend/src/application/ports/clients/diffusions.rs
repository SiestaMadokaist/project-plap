use std::rc::Rc;

use domain::{commands::inference::InferenceConfig, errors::DomainError};
use pkg::comfy;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait DiffusionClient {
    async fn generate(
        &self,
        params: &InferenceConfig<String>,
        workflow: Option<comfy::workflow::api::ExportApi>,
    ) -> Result<(), DomainError>;
}

// Lets a `Box<dyn DiffusionClient>` be built from an `Rc<T>` without cloning `T`
// itself — needed when the same client instance is shared (e.g. a mock with
// expectations set once, boxed on every `diffusion()` call).
#[async_trait::async_trait(?Send)]
impl<T: DiffusionClient> DiffusionClient for Rc<T> {
    async fn generate(
        &self,
        params: &InferenceConfig<String>,
        workflow: Option<comfy::workflow::api::ExportApi>,
    ) -> Result<(), DomainError> {
        self.as_ref().generate(params, workflow).await
    }
}
