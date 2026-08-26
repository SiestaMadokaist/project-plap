use domain::commands::compute::{ComputeInstanceID, ComputeRegion};
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait ComputeAgent {
    async fn ip(&self) -> anyhow::Result<String>;
    async fn region(&self) -> anyhow::Result<ComputeRegion>;
    async fn instance_id(&self) -> anyhow::Result<ComputeInstanceID>;
}
