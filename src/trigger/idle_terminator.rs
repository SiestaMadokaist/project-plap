use crate::{
    application::usecases::agent::{manage_compute::ManageCompute, traits::AgentClients},
    domain::commands::compute::{ComputeArgs, ComputeCommand},
    pkg::types::{
        peek::Peek,
        time::{Second, Timestamp},
    },
};
use std::rc::Rc;

/**
 * auto terminator that check if last_ok timestamp has passed its threshold
 * it'll automatically request to shutdown the compute instance
 */
pub struct IdleTerminator<C: AgentClients> {
    clients: Rc<C>,
    last_active: Peek<Timestamp>,
    tolerance: Second,
    interval: Second,
}

impl<C: AgentClients> IdleTerminator<C> {
    pub fn new(
        clients: Rc<C>,
        start_at: Peek<Timestamp>,
        tolerance: Second,
        interval: Second,
    ) -> Self {
        Self {
            clients,
            last_active: start_at,
            tolerance,
            interval,
        }
    }

    // call aws api using own IP
    async fn instance_id(&self) -> anyhow::Result<String> {
        todo!();
    }

    fn compute_args(&self, instance_id: String) -> ComputeArgs {
        let command = ComputeCommand::Stop;
        ComputeArgs {
            instance_id,
            command,
        }
    }

    async fn on_interval(&self) -> anyhow::Result<()> {
        let tolerance = &self.tolerance;
        let now = Timestamp::now();
        let last_ok = &self.last_active.get();
        let delta = now.sub(last_ok);
        if !delta.gt(tolerance) {
            return Ok(());
        }
        let id = self.instance_id().await?;
        let args = self.compute_args(id);
        let manage = ManageCompute::new(self.clients.clone(), args);
        manage.exec().await?;
        Ok(())
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(self.interval.0 as u64)).await;
            self.on_interval().await?;
        }
    }
}
