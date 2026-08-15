use crate::{
    application::ports::repository::error::RepositoryError,
    domain::commands::command::{ActionId, CommandDomain, Progression},
};

pub type AgentCommandError = RepositoryError<ActionId>;

#[allow(async_fn_in_trait)]
pub trait AgentCommandRepository {
    async fn insert(&self, command: CommandDomain) -> Result<ActionId, AgentCommandError>;
    async fn in_progress(&self, limit: i32) -> Result<Vec<CommandDomain>, AgentCommandError>;
    async fn get(&self, id: &ActionId) -> Result<CommandDomain, AgentCommandError>;
    async fn set_progress(
        &self,
        id: &ActionId,
        progress: &Progression,
    ) -> Result<CommandDomain, AgentCommandError>;
}
