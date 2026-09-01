use crate::application::{
    ports::repository::agent_command::AgentCommandRepository,
    usecases::agent::command_handler::{
        CommandHandler, CommandHandlerClients, CommandHandlerRepos,
    },
};
use domain::commands::command::CommandStage;
use pkg::types::time::Second;

pub struct CommandQ<'a, R: CommandHandlerRepos, C: CommandHandlerClients> {
    clients: &'a C,
    repos: &'a R,
    interval: Second,
}

impl<'a, R: CommandHandlerRepos, C: CommandHandlerClients> CommandQ<'a, R, C> {
    pub fn new(clients: &'a C, repos: &'a R, interval: Second) -> Self {
        Self {
            clients,
            repos,
            interval,
        }
    }

    async fn on_interval(&self) -> anyhow::Result<()> {
        let loader = self.repos.agent_command();
        // load 1 at a time, so that if priority is changed, the effect immediately applied
        // even if the task only partially done.
        let in_progress = loader.by_stage(CommandStage::InProgress, 1).await?;
        if !in_progress.is_empty() {
            tracing::info!("found {} in progress command", in_progress.len());
        }
        // "lowest" priority score first
        // lowest mean earliest being put...
        // or, just some command with higher urgency.
        for command in in_progress.into_iter() {
            let mut handler = CommandHandler::new(self.repos, self.clients, command);
            handler.exec().await?;
        }
        Ok(())
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            let interval = &self.interval;
            tokio::time::sleep(interval.to_duration()).await;
            self.on_interval().await?;
        }
    }
}
