use std::rc::Rc;

use crate::{
    application::{
        ports::repository::agent_command::AgentCommandRepository,
        usecases::agent::{
            inference::RunInference,
            traits::{AgentClients, AgentRepos},
        },
    },
    domain::commands::{
        command::{Action::Inference, CommandDomain, Progression},
        inference::InferenceArgs,
    },
};

pub struct CommandHandler<R: AgentRepos, C: AgentClients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: CommandDomain,
}

impl<R: AgentRepos, C: AgentClients> CommandHandler<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: CommandDomain) -> Self {
        CommandHandler {
            repo,
            client,
            params,
        }
    }

    async fn record_progress(&self, progress: &Option<Progression>) -> anyhow::Result<()> {
        match progress {
            None => Ok(()),
            Some(p) => {
                let agent_repo = self.repo.agent_command();
                let id = &self.params.action_id;
                if let Err(x) = agent_repo.set_progress(id, p).await {
                    Err(x.into())
                } else {
                    Ok(())
                }
            }
        }
    }

    async fn handle_inference(&self, arg: &InferenceArgs) -> anyhow::Result<()> {
        let progress = self.params.progress.clone();
        let config = &arg.config;
        let client = self.client.clone();
        let mut inferer = RunInference::new(client, progress, config);
        let updated_progress = inferer.generate().await?;
        self.record_progress(&updated_progress).await?;
        Ok(())
    }

    pub async fn exec(&mut self) -> anyhow::Result<()> {
        let action = &self.params.action;
        match action {
            Inference(arg) => self.handle_inference(&arg).await,
            _ => Ok(()),
        }
    }
}
