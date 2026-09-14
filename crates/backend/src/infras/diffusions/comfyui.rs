use domain::{commands::inference::InferenceConfig, errors::DomainError};
use pkg::{comfy, types::strings::URL};

use crate::application::ports::clients::diffusions::DiffusionClient;

pub struct ComfyUI {
    base_url: URL,
    client: reqwest::Client,
}

impl ComfyUI {
    pub fn new(base_url: URL) -> Self {
        let client = reqwest::Client::new();
        Self { base_url, client }
    }
}

#[async_trait::async_trait(?Send)]
impl DiffusionClient for ComfyUI {
    async fn generate(
        &self,
        params: &InferenceConfig<String>,
        workflow: Option<comfy::workflow::PostRequests>,
    ) -> Result<(), DomainError> {
        if workflow.is_none() {
            return Err(DomainError::ValidationError(
                "workflow cannot be None".into(),
            ));
        }
        let mut postrequest = workflow.unwrap();
        postrequest.set_checkpoint(&params.checkpoint);
        postrequest.set_positive(&params.positive);
        postrequest.set_resolution(params.width, params.height);
        let body = serde_json::to_string_pretty(&postrequest)
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        let client = &self.client;
        let url = self.base_url.e("/api/prompt");
        let resp = client
            .post(url.0)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?;
        let data = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|x| DomainError::Serialize(x.to_string()))?;
        tracing::info!("data: {}", data);
        Err(DomainError::EmptyResponse)
    }
}
