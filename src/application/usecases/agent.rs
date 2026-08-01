use std::rc::Rc;
pub mod inference;
pub mod traits;

use crate::{
    application::usecases::agent::{
        inference::RunInference,
        traits::{AgentClients, AgentRepos},
    },
    domain::commands::command::{Action::Inference, CommandDomain},
};

pub struct AgentConsumer<R: AgentRepos, C: AgentClients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: CommandDomain,
}

impl<R: AgentRepos, C: AgentClients> AgentConsumer<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: CommandDomain) -> Self {
        AgentConsumer {
            repo,
            client,
            params,
        }
    }

    pub fn exec(&self) -> anyhow::Result<()> {
        let action = &self.params.action;
        match action {
            Inference(arg) => {
                let progress = arg.progress.clone();
                let config = &arg.config;
                let client = self.client.clone();
                let inferer = RunInference::new(client, progress, config);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
