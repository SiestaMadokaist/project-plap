#[allow(async_fn_in_trait)]
pub trait ComputeClient {
    async fn stop(&self) -> anyhow::Result<()>;
    async fn launch(&self) -> anyhow::Result<()>;
    async fn terminate(&self) -> anyhow::Result<()>;
    async fn reboot(&self) -> anyhow::Result<()>;
}
