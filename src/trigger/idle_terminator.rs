use crate::{
    application::{
        ports::clients::{compute::ComputeEngines, compute_agent::ComputeAgent},
        usecases::agent::{
            manage_compute::ManageCompute,
            traits::{AgentClients, AgentRepos},
        },
    },
    domain::commands::compute::{ComputeArgs, ComputeCommand, ComputeInstanceID, ComputeRegion},
    pkg::types::{
        peek::Peek,
        time::{Second, Timestamp},
    },
};
use std::{cell::OnceCell, rc::Rc};
pub struct Memo {
    ip: OnceCell<String>,
    instance_id: OnceCell<ComputeInstanceID>,
    action: OnceCell<ComputeCommand>,
    document: OnceCell<()>,
}

/**
 * auto terminator that check if last_ok timestamp has passed its threshold
 * it'll automatically request to shutdown the compute instance
 */
pub struct IdleTerminator<C: AgentClients, R: AgentRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    last_active: Peek<Timestamp>,
    tolerance: Second,
    interval: Second,
    memo: Memo,
}

impl<C: AgentClients, R: AgentRepos> IdleTerminator<C, R> {
    pub fn new(
        clients: Rc<C>,
        repos: Rc<R>,
        start_at: Peek<Timestamp>,
        tolerance: Second,
        interval: Second,
    ) -> Self {
        Self {
            clients,
            repos,
            last_active: start_at,
            tolerance,
            interval,
            memo: Memo {
                ip: OnceCell::new(),
                instance_id: OnceCell::new(),
                action: OnceCell::new(),
                document: OnceCell::new(),
            },
        }
    }

    // async fn ip(&self) -> anyhow::Result<String> {
    //     let agent = self.clients.agent();
    //     let ip = agent.ip().await?;
    //     Ok(ip)
    // }

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
        let command = self.memo.action.get_or_init(|| {
            // let table = self.repos.bi
            todo!()
        });
        Ok(command)
    }

    async fn compute_args(&self) -> anyhow::Result<ComputeArgs> {
        let command = self.action().await?;
        let instance_id = self.instance_id().await?;
        let region = self.region().await?;
        let args = ComputeArgs {
            region: region.clone(),
            instance_id: instance_id.clone(),
            command: command.clone(),
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
        let manage = ManageCompute::new(self.clients.clone(), args);
        manage.exec().await.expect("failed to terminate instance");
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
