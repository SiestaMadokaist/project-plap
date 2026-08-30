use crate::application::ports::repository::error::RepositoryError;
use domain::commands::command::{ActionId, CommandDomain, CommandStage, Progression};

pub type AgentCommandError = RepositoryError<ActionId>;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait AgentCommandRepository {
    async fn insert(&self, command: CommandDomain) -> Result<ActionId, AgentCommandError>;
    async fn by_stage(
        &self,
        stage: CommandStage,
        limit: i32,
    ) -> Result<Vec<CommandDomain>, AgentCommandError>;
    async fn get(&self, id: &ActionId) -> Result<CommandDomain, AgentCommandError>;
    async fn set_progress(
        &self,
        id: &ActionId,
        progress: &Progression,
    ) -> Result<CommandDomain, AgentCommandError>;

    async fn delete(&self, id: &ActionId) -> Result<(), AgentCommandError>;
}
