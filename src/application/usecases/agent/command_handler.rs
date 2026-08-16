use std::rc::Rc;

#[cfg(feature = "datatransfer")]
// use crate::infras::civitai::{self, dto::model_version::ModelVersionDTO};
use crate::{
    application::{
        ports::{
            clients::{self},
            repository::{self, agent_command::AgentCommandRepository},
        },
        usecases::agent::h_inference::HandleInference,
    },
    domain::{
        commands::{
            command::{
                Action::{Inference, Network},
                CommandDomain, CommandStage, Progression,
            },
            inference::InferenceArgs,
            network::NetworkArgs,
        },
        errors::DomainError,
    },
    pkg::macros::{trait_clients, trait_repos},
};

trait_clients!(
    CommandHandlerClients,
    clients::container::HasDiffusion,
    clients::container::HasModelStorage,
    clients::container::HasCivitai
);
trait_repos!(CommandHandlerRepos, repository::container::HasAgentCommand);

pub struct CommandHandler<R: CommandHandlerRepos, C: CommandHandlerClients> {
    repo: Rc<R>,
    clients: Rc<C>,
    params: CommandDomain,
}

impl<R: CommandHandlerRepos, C: CommandHandlerClients> CommandHandler<R, C> {
    pub fn new(repo: Rc<R>, clients: Rc<C>, params: CommandDomain) -> Self {
        CommandHandler {
            repo,
            clients,
            params,
        }
    }

    async fn record_progress(&self, progress: &Progression) -> Result<CommandStage, DomainError> {
        let agent_repo = self.repo.agent_command();
        let id = &self.params.action_id;
        let updated = agent_repo
            .set_progress(id, progress)
            .await
            .map_err(|x| DomainError::HttpConnectionFailed(x.to_string()))?;
        if updated.progress.is_done() {
            Ok(CommandStage::Completed)
        } else {
            Ok(CommandStage::InProgress)
        }
    }

    #[cfg(feature = "datatransfer")]
    async fn handle_network(&self, args: &NetworkArgs) -> Result<CommandStage, DomainError> {
        use crate::application::usecases::agent::h_network::HandleNetwork;
        let handler = HandleNetwork::new(self.clients.clone(), args);
        handler.exec().await
    }

    async fn handle_inference(&self, arg: &InferenceArgs) -> Result<CommandStage, DomainError> {
        let progress = self.params.progress.clone();
        let config = &arg.config;
        let client = self.clients.clone();
        let mut handler = HandleInference::new(client, progress, config);
        let updated_progress = handler.exec().await?;
        let command_stage = self.record_progress(&updated_progress).await?;
        Ok(command_stage)
    }

    pub async fn exec(&mut self) -> Result<CommandStage, DomainError> {
        let action = &self.params.action;
        match action {
            Inference(arg) => self.handle_inference(arg).await,
            #[cfg(feature = "datatransfer")]
            Network(arg) => self.handle_network(arg).await,
            _ => Ok(CommandStage::Failed),
        }
    }
}
