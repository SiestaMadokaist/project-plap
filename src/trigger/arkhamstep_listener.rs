use std::{cell::Cell, rc::Rc};

use crate::{
    application::usecases::agent::traits::AgentRepos,
    pkg::types::time::{Second, Timestamp},
};

pub struct ArkhamStepListener<R: AgentRepos> {
    repos: Rc<R>,
    last_active: Rc<Cell<Timestamp>>,
    interval: Second,
}

impl<R: AgentRepos> ArkhamStepListener<R> {
    pub fn new(repos: Rc<R>, last_active: Rc<Cell<Timestamp>>, interval: Second) -> Self {
        Self {
            repos,
            last_active,
            interval,
        }
    }

    async fn get_last_step(&self) -> anyhow::Result<Timestamp> {
        todo!()
    }

    async fn on_interval(&self) -> anyhow::Result<()> {
        let last_step_at = self.get_last_step().await?;
        self.last_active.set(last_step_at);
        todo!()
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            let interval = &self.interval;
            tokio::time::sleep(interval.into()).await;
            self.on_interval().await?;
        }
    }
}
