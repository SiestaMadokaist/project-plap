use crate::application::usecases::agent::{
    h_compute::ManageCompute,
    traits::{AgentClients, AgentRepos},
};
use domain::commands::compute::{ComputeArgs, ComputeCommand, ComputeInstanceID, ComputeRegion};
use pkg::types::{
    peek::Peek,
    time::{Second, Timestamp},
};
use std::cell::OnceCell;
pub struct Memo {
    action: OnceCell<ComputeCommand>,
}

/**
 * auto terminator that check if last_ok timestamp has passed its threshold
 * it'll automatically request to shutdown the compute instance
 */
pub struct IdleTerminator<'a, C: AgentClients, R: AgentRepos> {
    clients: &'a C,
    _repos: &'a R,
    last_active: Peek<Timestamp>,
    tolerance: Second,
    interval: Second,
    memo: Memo,
}

impl<'a, C: AgentClients, R: AgentRepos> IdleTerminator<'a, C, R> {
    pub fn new(
        clients: &'a C,
        repos: &'a R,
        start_at: Peek<Timestamp>,
        tolerance: Second,
        interval: Second,
    ) -> Self {
        Self {
            clients,
            _repos: repos,
            last_active: start_at,
            tolerance,
            interval,
            memo: Memo {
                action: OnceCell::new(),
            },
        }
    }

    async fn instance_id(&self) -> anyhow::Result<ComputeInstanceID> {
        let agent = self.clients.agent();
        let id = agent.instance_id().await?;
        Ok(id)
    }

    async fn region(&self) -> anyhow::Result<ComputeRegion> {
        let agent = self.clients.agent();
        let region = agent.region().await?;
        Ok(region)
    }

    async fn action(&self) -> anyhow::Result<&ComputeCommand> {
        // read action to perform from dynamodb
        let command = self.memo.action.get_or_init(|| ComputeCommand::Stop);
        Ok(command)
    }

    async fn compute_args(&self) -> anyhow::Result<ComputeArgs> {
        let command = self.action().await?;
        let instance_id = self.instance_id().await?;
        let region = self.region().await?;
        let args = ComputeArgs {
            region,
            instance_id: instance_id.clone(),
            command: *command,
        };
        Ok(args)
    }

    async fn on_interval(&self) -> anyhow::Result<()> {
        let tolerance = &self.tolerance;
        let now = Timestamp::now();
        let last_ok = &self.last_active.get();
        let delta = now.sub(last_ok);
        if delta.lt(tolerance) {
            tracing::trace!("inactive for {} second tolerable", delta.0);
            return Ok(());
        }
        tracing::info!(
            "inactive for {} second, beyond tolerance of {} second, terminating",
            delta.0,
            tolerance.0
        );
        let args = self.compute_args().await?;
        let manage = ManageCompute::new(self.clients, args);
        // let termination = manage.exec().await;
        if let Err(x) = manage.exec().await {
            tracing::error!("termination failed with error: {}", x);
        }
        Ok(())
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            let interval = &self.interval;
            tokio::time::sleep(interval.into()).await;
            self.on_interval().await?;
        }
    }
}
