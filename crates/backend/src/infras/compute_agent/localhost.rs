use crate::application::ports::clients::compute_agent::ComputeAgent;
use domain::commands::compute::{ComputeInstanceID, ComputeRegion};
use pkg::auth::claims::Username;

pub struct LocalhostAgent {
    _name: String,
}

impl LocalhostAgent {
    pub fn new() -> Self {
        Self {
            _name: "localhost".into(),
        }
    }
}

impl Default for LocalhostAgent {
    fn default() -> Self {
        Self::new()
    }
}
#[async_trait::async_trait(?Send)]
impl ComputeAgent for LocalhostAgent {
    async fn ip(&self) -> anyhow::Result<String> {
        Ok("127.0.0.1".into())
    }
    async fn instance_id(&self) -> anyhow::Result<ComputeInstanceID> {
        let s: String = "localhost".into();
        Ok(s.into())
    }
    async fn region(&self) -> anyhow::Result<ComputeRegion> {
        Ok(ComputeRegion::AWSApSoutheast2)
    }
    // No instance/DescribeTags on localhost - read the same USERNAME a real box would
    // have gotten from its launch tag, so a local run can still hit HotReloadRepository.
    async fn username(&self) -> anyhow::Result<Username> {
        let name = std::env::var("USERNAME").unwrap_or_else(|_| "guest".into());
        Ok(Username(name))
    }
}
