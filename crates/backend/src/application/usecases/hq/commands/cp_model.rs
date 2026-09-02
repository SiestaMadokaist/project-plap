use crate::application::ports::repository::{
    agent_command::AgentCommandRepository, container::HasAgentCommand,
};
use domain::{commands::command::CommandDomain, errors::DomainError};
use dto::resources::commands::{CpPayload, CpResponse};
use pkg::macros::trait_repos;

trait_repos!(CPModelRepos, HasAgentCommand);

/**
 * obtain a model from a known remote service (e.g: civitai / s3)
 * and store it into remote storage service (e.g: s3) via the agent queue.
 */
pub struct CPModelSvc<'a, R: CPModelRepos> {
    repos: &'a R,
    payload: CpPayload,
}

impl<'a, R: CPModelRepos> CPModelSvc<'a, R> {
    pub fn new(repos: &'a R, payload: CpPayload) -> Self {
        Self { repos, payload }
    }

    pub async fn exec(&self) -> Result<CpResponse, DomainError> {
        self.run().await
    }

    async fn run(&self) -> Result<CpResponse, DomainError> {
        let repo = self.repos.agent_command();
        let command = CommandDomain::network(
            self.payload.action_id.clone(),
            self.payload.args.clone(),
            self.payload.priority,
        );
        tracing::debug!("command: {}", serde_json::to_value(&command)?);
        repo.insert(command.clone())
            .await
            .map_err(|x| DomainError::ApiError(x.to_string()))?;
        Ok(CpResponse { command })
    }
}
