use crate::application::{
    ports::{
        clients::{self},
        repository::{self, agent_command::AgentCommandRepository},
    },
    usecases::agent::h_inference::HandleInference,
};
#[cfg(feature = "datatransfer")]
use domain::commands::network::NetworkArgs;
use domain::{
    commands::{
        command::{Action, CommandDomain, CommandStage, Progression},
        inference::InferenceArgs,
    },
    errors::DomainError,
};
use pkg::macros::{trait_clients, trait_repos};

trait_clients!(
    CommandHandlerClients,
    clients::container::HasDiffusion,
    clients::container::HasModelStorage,
    clients::container::HasInferenceModelProvider,
    clients::container::HasNotification
);
trait_repos!(CommandHandlerRepos, repository::container::HasAgentCommand);

pub struct CommandHandler<'a, R: CommandHandlerRepos, C: CommandHandlerClients> {
    repo: &'a R,
    clients: &'a C,
    params: CommandDomain,
}

impl<'a, R: CommandHandlerRepos, C: CommandHandlerClients> CommandHandler<'a, R, C> {
    pub fn new(repo: &'a R, clients: &'a C, params: CommandDomain) -> Self {
        CommandHandler {
            repo,
            clients,
            params,
        }
    }

    async fn record_progress(&self, progress: &Progression) -> Result<CommandStage, DomainError> {
        let agent_repo = self.repo.agent_command();
        let id = &self.params.action_id;
        tracing::info!("recording progress: {}", id);
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
        use crate::application::{
            ports::clients::notification::NotificationClient,
            usecases::agent::h_network::HandleNetwork,
        };
        let progress = self.params.progress.clone();
        let handler = HandleNetwork::new(self.clients, args, progress);
        let updated = handler.exec().await;
        let updated_json = serde_json::to_string_pretty(&updated)?;
        tracing::info!("recording updated progress: ```\n{}\n```", &updated_json);
        let command_stage = self.record_progress(&updated).await?;
        let json = serde_json::to_string_pretty(args)?;
        let message = format!("network request is {}.\n```{}```", command_stage, json);
        let notifier = self.clients.notification();
        notifier.notify(&message).await?;
        Ok(command_stage)
    }

    async fn handle_inference(&self, arg: &InferenceArgs) -> Result<CommandStage, DomainError> {
        let progress = self.params.progress.clone();
        let config = &arg.config;
        let mut handler = HandleInference::new(self.clients, progress, config);
        let updated_progress = handler.exec().await;
        let command_stage = self.record_progress(&updated_progress).await?;
        Ok(command_stage)
    }

    pub async fn exec(&mut self) -> Result<CommandStage, DomainError> {
        let action = &self.params.action;
        match action {
            Action::Inference(arg) => self.handle_inference(arg).await,
            #[cfg(feature = "datatransfer")]
            Action::Network(arg) => self.handle_network(arg).await,
            _ => Ok(CommandStage::Failed),
        }
    }
}
