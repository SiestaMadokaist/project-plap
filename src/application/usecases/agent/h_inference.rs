use std::rc::Rc;

use crate::{
    application::ports::clients::container::HasDiffusion,
    domain::{
        commands::{command::Progression, inference::InferenceConfig},
        errors::DomainError,
    },
    pkg::macros::trait_clients,
};

trait_clients!(HandleInferenceClient, HasDiffusion);

pub struct HandleInference<'a, C: HandleInferenceClient> {
    clients: Rc<C>,
    progress: Progression,
    config: &'a InferenceConfig<String>,
}

impl<'a, C: HandleInferenceClient> HandleInference<'a, C> {
    pub fn new(clients: Rc<C>, progress: Progression, config: &'a InferenceConfig<String>) -> Self {
        Self {
            clients,
            progress,
            config,
        }
    }

    pub async fn generate(&mut self) -> Result<Progression, DomainError> {
        if self.progress.is_done() {
            return Ok(self.progress);
        }
        let diffusion = self.clients.diffusion();
        diffusion
            .generate(self.config)
            .await
            .map_err(|x| DomainError::ApiError(x.to_string()))?;
        self.progress.increment();
        Ok(self.progress)
    }

    pub async fn exec(&mut self) -> Result<Progression, DomainError> {
        self.generate().await
    }
}
