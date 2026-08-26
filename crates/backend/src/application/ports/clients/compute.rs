use domain::{
    commands::compute::{ComputeInstanceID, ComputeRegion},
    errors::DomainError,
};
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait ComputeEngine {
    async fn stop(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    async fn launch(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    async fn terminate(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    async fn reboot(&self, id: &ComputeInstanceID) -> Result<(), DomainError>;
    fn region(&self) -> ComputeRegion;
}

pub trait ComputeEngines {
    type Engine: ComputeEngine;
    fn get(&self, region: &ComputeRegion) -> Option<Self::Engine>;
}
