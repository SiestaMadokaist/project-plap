use crate::domain::commands::compute::{ComputeInstanceID, ComputeRegion};

#[allow(async_fn_in_trait)]
pub trait ComputeEngine {
    async fn stop(&self, id: &ComputeInstanceID) -> anyhow::Result<()>;
    async fn launch(&self, id: &ComputeInstanceID) -> anyhow::Result<()>;
    async fn terminate(&self, id: &ComputeInstanceID) -> anyhow::Result<()>;
    async fn reboot(&self, id: &ComputeInstanceID) -> anyhow::Result<()>;
    fn region(&self) -> ComputeRegion;
}

pub trait ComputeEngines {
    type Engine: ComputeEngine;
    fn get(&self, region: &ComputeRegion) -> Option<Self::Engine>;
}
