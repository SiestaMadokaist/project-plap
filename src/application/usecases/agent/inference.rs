use std::rc::Rc;

use anyhow::Ok;

use crate::{
    application::usecases::agent::AgentClients,
    domain::commands::inference::{InferenceConfig, InferenceProgress},
};

pub struct RunInference<'a, C: AgentClients> {
    clients: Rc<C>,
    progress: InferenceProgress,
    config: &'a InferenceConfig,
}

impl<'a, C: AgentClients> RunInference<'a, C> {
    pub fn new(clients: Rc<C>, progress: InferenceProgress, config: &'a InferenceConfig) -> Self {
        RunInference {
            clients,
            progress,
            config,
        }
    }

    pub async fn exec(&self) -> anyhow::Result<()> {
        Ok(()) // todo
    }
}
