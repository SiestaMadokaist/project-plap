use crate::{
    application::ports::repository::error::RepositoryError,
    domain::commands::command::CommandDomain,
    pkg::types::{id::ActionID, progress::Progression},
};

#[derive(Debug, thiserror::Error)]
pub enum AgentCommandError {
    #[error(transparent)]
    Repo(#[from] RepositoryError),
}

pub trait AgentCommandRepository {
    async fn insert(&self, command: CommandDomain)
        -> Result<Vec<CommandDomain>, AgentCommandError>;
    async fn in_progress(&self) -> Result<Vec<CommandDomain>, AgentCommandError>;
    async fn get(&self, id: &ActionID) -> Result<CommandDomain, AgentCommandError>;
    async fn set_progress(
        &self,
        id: &ActionID,
        progress: &Progression,
    ) -> Result<CommandDomain, AgentCommandError>;
}
