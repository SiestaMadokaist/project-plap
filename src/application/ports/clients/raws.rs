use crate::domain::translation::{ChapterId, NovelId};
#[derive(Debug, thiserror::Error)]
pub enum RawsError {
    #[error("failed")]
    Failed,
}

#[allow(async_fn_in_trait)]
pub trait RawsClient {
    async fn latest(&self, id: &NovelId) -> Result<ChapterId, RawsError>;
    async fn read(&self, id: &ChapterId) -> Result<String, RawsError>;
}
