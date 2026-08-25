use std::rc::Rc;

use crate::{
    application::ports::repository::{
        agent_command::AgentCommandRepository, container::HasAgentCommand,
    },
    domain::{
        commands::command::{CommandDomain, CommandStage},
        errors::DomainError,
    },
    pkg::macros::trait_repos,
};

trait_repos!(IRepos, HasAgentCommand);

pub struct Payload {
    stage: CommandStage,
    limit: i32,
}
pub struct GetList<R: IRepos> {
    repos: Rc<R>,
    payload: Payload,
}

impl<R: IRepos> GetList<R> {
    pub fn new(repos: Rc<R>, payload: Payload) -> Self {
        Self { repos, payload }
    }

    pub async fn exec(&self) -> Result<Vec<CommandDomain>, DomainError> {
        let command_repo = self.repos.agent_command();
        let payload = &self.payload;
        let in_progress = command_repo
            .by_stage(payload.stage, payload.limit)
            .await
            .map_err(|x| DomainError::Disconnected(x.to_string()))?;
        Ok(in_progress)
    }
}
