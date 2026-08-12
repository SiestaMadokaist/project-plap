use std::rc::Rc;

use anyhow::Ok;

use crate::{
    application::ports::clients::container::HasDiffusion,
    domain::commands::{command::Progression, inference::InferenceConfig},
    pkg::macros::trait_clients,
};

trait_clients!(RunInferenceClient, HasDiffusion);

pub struct RunInference<'a, C: RunInferenceClient> {
    clients: Rc<C>,
    progress: Progression,
    config: &'a InferenceConfig,
}

impl<'a, C: RunInferenceClient> RunInference<'a, C> {
    pub fn new(clients: Rc<C>, progress: Progression, config: &'a InferenceConfig) -> Self {
        Self {
            clients,
            progress,
            config,
        }
    }

    pub async fn generate(&mut self) -> anyhow::Result<Option<Progression>> {
        if self.progress.is_done() {
            return Ok(None);
        }
        let diffusion = self.clients.diffusion();
        diffusion.generate(self.config).await?;
        self.progress.increment();
        Ok(Some(self.progress))
    }

    pub async fn exec(&mut self) -> anyhow::Result<()> {
        self.generate().await?;
        Ok(())
    }
}
