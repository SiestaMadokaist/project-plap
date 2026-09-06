use domain::{
    commands::compute::{ComputeInstance, ComputeInstanceID, ComputeRegion, LaunchConfig},
    errors::DomainError,
};
use pkg::auth::claims::Username;
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait ComputeEngine {
    async fn stop(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    async fn start(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    async fn terminate(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    async fn reboot(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    /// `config` is looked up by the caller (HotReloadRepository::launch_config), not
    /// stored on the engine — a launch is always for whichever user requested it.
    /// `username` is stamped on the instance as a tag, so the agent that boots on
    /// it can read its own identity back via `ComputeAgent::username`. `spot`
    /// explicitly requests spot capacity when `true`; when `false` the instance
    /// launches however the launch template itself is configured (see the note on
    /// `EC2::launch` — there is no API-level way to force on-demand over a
    /// template that already defaults to spot).
    async fn launch(
        &self,
        config: &LaunchConfig,
        username: &Username,
        script: &Option<String>,
        spot: bool,
    ) -> Result<ComputeInstance, DomainError>;
    async fn list(&self) -> Result<Vec<ComputeInstance>, DomainError>;
    async fn open(&self, ip: &str) -> Result<(), DomainError>;
    fn region(&self) -> ComputeRegion;
}

pub trait ComputeEngines {
    type Engine: ComputeEngine;
    fn get(&self, region: &ComputeRegion) -> Option<Self::Engine>;
}
