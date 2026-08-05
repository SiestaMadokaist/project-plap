use crate::{
    application::ports::repository::error::RepositoryError,
    domain::commands::command::{ActionID, CommandDomain, Progression},
};

pub type AgentCommandError = RepositoryError<ActionID>;

#[allow(async_fn_in_trait)]
pub trait AgentCommandRepository {
    async fn insert(&self, command: CommandDomain) -> Result<ActionID, AgentCommandError>;
    async fn in_progress(&self, limit: i32) -> Result<Vec<CommandDomain>, AgentCommandError>;
    async fn get(&self, id: &ActionID) -> Result<CommandDomain, AgentCommandError>;
    async fn set_progress(
        &self,
        id: &ActionID,
        progress: &Progression,
    ) -> Result<CommandDomain, AgentCommandError>;
}
