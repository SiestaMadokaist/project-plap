use std::rc::Rc;

use anyhow::Ok;

use crate::{
    application::usecases::agent::traits::AgentClients,
    domain::commands::{command::Progression, inference::InferenceConfig},
};

pub struct RunInference<'a, C: AgentClients> {
    clients: Rc<C>,
    progress: Progression,
    config: &'a InferenceConfig,
}

impl<'a, C: AgentClients> RunInference<'a, C> {
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
