use crate::application::ports::clients::compute_agent::ComputeAgent;
use domain::commands::compute::{ComputeInstanceID, ComputeRegion};

pub struct LocalhostAgent {
    name: String,
}

impl LocalhostAgent {
    pub fn new() -> Self {
        Self {
            name: "localhost".into(),
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
}
