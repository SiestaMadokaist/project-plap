use std::rc::Rc;

use crate::{
    application::{
        ports::repository::agent_command::AgentCommandRepository,
        usecases::agent::command_handler::{
            CommandHandler, CommandHandlerClients, CommandHandlerRepos,
        },
    },
    pkg::types::time::Second,
};

pub struct CommandQ<R: CommandHandlerRepos, C: CommandHandlerClients> {
    clients: Rc<C>,
    repos: Rc<R>,
    interval: Second,
}

impl<R: CommandHandlerRepos, C: CommandHandlerClients> CommandQ<R, C> {
    pub fn new(clients: Rc<C>, repos: Rc<R>, interval: Second) -> Self {
        Self {
            clients,
            repos,
            interval,
        }
    }

    async fn on_interval(&self) -> anyhow::Result<()> {
        let loader = self.repos.agent_command();
        // could just load 1 at a time, but idk.
        let in_progress = loader.in_progress(3).await?;
        // "lowest" priority score first
        // lowest mean earliest being put...
        // or, just some command with higher urgency.
        let next = in_progress.into_iter().next();
        match next {
            None => Ok(()),
            Some(params) => {
                let mut handler =
                    CommandHandler::new(self.repos.clone(), self.clients.clone(), params);
                let result = handler.exec().await?;
                Ok(result)
            }
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            let interval = &self.interval;
            tokio::time::sleep(interval.into()).await;
            self.on_interval().await?
        }
    }
}
