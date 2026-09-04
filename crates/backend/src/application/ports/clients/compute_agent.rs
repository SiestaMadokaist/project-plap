use domain::commands::compute::{ComputeInstanceID, ComputeRegion};
use pkg::auth::claims::Username;
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait ComputeAgent {
    async fn ip(&self) -> anyhow::Result<String>;
    async fn region(&self) -> anyhow::Result<ComputeRegion>;
    async fn instance_id(&self) -> anyhow::Result<ComputeInstanceID>;
    /// Who launched this instance, read back from its own `Username` tag
    /// (set by `EC2::launch` at `RunInstances` time via `ec2:DescribeTags`).
    async fn username(&self) -> anyhow::Result<Username>;
}
