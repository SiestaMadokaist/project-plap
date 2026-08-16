use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    application::ports::repository::{
        agent_command::AgentCommandRepository, container::HasAgentCommand,
    },
    domain::{
        commands::{
            command::{ActionId, CommandDomain},
            network::NetworkArgs,
        },
        errors::DomainError,
    },
    json_type,
    pkg::macros::trait_repos,
};

/**
 * obtain model from a known remote service (e.g: civitai)
 * store it into remote storage service (e.g: s3)
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    action_id: ActionId,
    args: NetworkArgs,
    priority: u64,
}
json_type!(Payload);
trait_repos!(IRepos, HasAgentCommand);
pub struct CPModel<R: IRepos> {
    repos: Rc<R>,
    payload: Payload,
}

impl<R: IRepos> CPModel<R> {
    pub fn new(repos: Rc<R>, payload: Payload) -> Self {
        Self { repos, payload }
    }

    pub async fn exec(&self) -> Result<serde_json::Value, DomainError> {
        let result = self.run().await?;
        let v = serde_json::to_value(result).map_err(|x| DomainError::Serialize(x.to_string()));
        v
    }

    async fn run(&self) -> Result<ActionId, DomainError> {
        let repo = self.repos.agent_command();
        let command = CommandDomain::network(
            self.payload.action_id.clone(),
            self.payload.args.clone(),
            self.payload.priority,
        );
        tracing::debug!("command: {}", serde_json::to_value(&command)?);
        let result = repo
            .insert(command)
            .await
            .map_err(|x| DomainError::ApiError(x.to_string()));
        result
    }
}
