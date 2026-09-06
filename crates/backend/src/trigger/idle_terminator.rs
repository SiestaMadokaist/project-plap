use crate::application::{
    ports::repository::hot_reload::HotReloadRepository,
    usecases::agent::{
        h_compute::ManageCompute,
        traits::{AgentClients, AgentRepos},
    },
};
use chrono::Utc;
use domain::{
    commands::compute::{ComputeArgs, ComputeInstanceID, ComputeRegion},
    errors::DomainError,
    hot_reload::BillOptimization,
};
use pkg::types::{
    peek::Peek,
    time::{Second, Timestamp},
};
use std::{cell::RefCell, ops::Sub, time::Duration};

/// How long a fetched [`BillOptimization`] is trusted before `optimization()` goes
/// back to HotReloadRepository for a fresh one.
struct Cache {
    ttl: Second,
    bill_optimization: RefCell<Option<(BillOptimization, chrono::DateTime<Utc>)>>,
}

impl Cache {
    fn new(ttl: Second) -> Self {
        Self {
            ttl,
            bill_optimization: RefCell::new(None),
        }
    }

    /// `Some` only while the last fetch is still within `ttl`.
    fn get(&self) -> Option<BillOptimization> {
        match &*self.bill_optimization.borrow() {
            Some((value, expires_at)) if Utc::now() < *expires_at => Some(*value),
            _ => None,
        }
    }

    fn set(&self, value: BillOptimization) {
        let expires_at = Utc::now() + self.ttl.clone().to_delta();
        *self.bill_optimization.borrow_mut() = Some((value, expires_at));
    }
}

/// When `optimization()` can't reach HotReloadRepository, retry on this cadence
/// instead of crashing the loop (and taking queue_handler/output_listener down
/// with it via `tokio::try_join!` in main).
const FALLBACK_POLL: Duration = Duration::from_secs(30);

/**
 * auto terminator that check if last_ok timestamp has passed its threshold
 * it'll automatically request to shutdown the compute instance
 */
pub struct IdleTerminator<'a, C: AgentClients, R: AgentRepos> {
    clients: &'a C,
    repos: &'a R,
    last_active: Peek<Timestamp>,
    cache: Cache,
}

impl<'a, C: AgentClients, R: AgentRepos> IdleTerminator<'a, C, R> {
    pub fn new(clients: &'a C, repos: &'a R, start_at: Peek<Timestamp>, cache_ttl: Second) -> Self {
        Self {
            clients,
            repos,
            last_active: start_at,
            cache: Cache::new(cache_ttl),
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

    /// The bill-optimization config for whoever launched this instance (read back
    /// from its own `Username` tag - see `ComputeAgent::username`), served from
    /// cache while fresh and refetched from HotReloadRepository once `cache_ttl`
    /// has passed.
    async fn optimization(&self) -> Result<BillOptimization, DomainError> {
        if let Some(cached) = self.cache.get() {
            return Ok(cached);
        }
        let username = self
            .clients
            .agent()
            .username()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;
        let reload = self.repos.hotreload();
        let fresh = reload.bill_optimization(&username).await?;
        self.cache.set(fresh);
        Ok(fresh)
    }

    async fn compute_args(&self) -> anyhow::Result<ComputeArgs> {
        let optimization = self.optimization().await?;
        let command = &optimization.action;
        let instance_id = self.instance_id().await?;
        let region = self.region().await?;
        let args = ComputeArgs {
            region,
            instance_id: instance_id.clone(),
            command: *command,
        };
        Ok(args)
    }

    async fn on_interval(&self) -> Result<(), DomainError> {
        let optimization = self.optimization().await?;
        let tolerance = &optimization.idle_tolerance;
        let now = Timestamp::now();
        let last_ok = &self.last_active.get();
        let now_utc = now.utc().expect("should be a valid utc");
        let last_ok_utc = last_ok.utc().expect("should be a valid utc");
        let delta = now_utc.sub(last_ok_utc);
        if delta.lt(&tolerance.to_delta()) {
            tracing::trace!("inactive for {} second tolerable", delta);
            return Ok(());
        }
        tracing::info!(
            "inactive for {} second, beyond tolerance of {} second, terminating",
            delta,
            tolerance.0
        );
        let args = self
            .compute_args()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;
        let manage = ManageCompute::new(self.clients, args);
        if let Err(x) = manage.exec().await {
            tracing::error!("termination failed with error: {}", x);
        }
        Ok(())
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            let sleep_for = match self.optimization().await {
                Ok(optimization) => optimization.check_interval.to_duration(),
                Err(e) => {
                    tracing::error!(
                        "failed to load bill-optimization config, retrying in {}s: {}",
                        FALLBACK_POLL.as_secs(),
                        e
                    );
                    FALLBACK_POLL
                }
            };
            tokio::time::sleep(sleep_for).await;
            if let Err(e) = self.on_interval().await {
                tracing::error!("idle-terminator tick failed: {}", e);
            }
        }
    }
}
