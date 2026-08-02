use aws_sdk_dynamodb::Client;

use crate::{
    application::ports::repository::agent_command::{AgentCommandError, AgentCommandRepository},
    domain::commands::command::CommandDomain,
    pkg::types::{id::ActionID, progress::Progression},
};

pub struct DDBAgentCommandRepository {
    client: Client,
    table: String,
}

impl DDBAgentCommandRepository {
    pub fn new(client: Client, table: String) -> Self {
        Self { client, table }
    }
}

impl AgentCommandRepository for DDBAgentCommandRepository {
    async fn insert(
        &self,
        _command: CommandDomain,
    ) -> Result<Vec<CommandDomain>, AgentCommandError> {
        let _ = (&self.client, &self.table);
        todo!()
    }

    async fn in_progress(&self) -> Result<Vec<CommandDomain>, AgentCommandError> {
        todo!()
    }

    async fn get(&self, _id: &ActionID) -> Result<CommandDomain, AgentCommandError> {
        todo!()
    }

    async fn set_progress(
        &self,
        _id: &ActionID,
        _progress: &Progression,
    ) -> Result<CommandDomain, AgentCommandError> {
        todo!()
    }
}
