use domain::{
    commands::compute::{ComputeInstanceID, ComputeRegion},
    errors::DomainError,
};
use pkg::auth::claims::Username;
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait(?Send)]
pub trait ComputeAgent {
    async fn ip(&self) -> Result<String, DomainError>;
    async fn region(&self) -> Result<ComputeRegion, DomainError>;
    async fn instance_id(&self) -> Result<ComputeInstanceID, DomainError>;
    /// Who launched this instance, read back from its own `Username` tag
    /// (set by `EC2::launch` at `RunInstances` time via `ec2:DescribeTags`).
    async fn username(&self) -> Result<Username, DomainError>;
}
