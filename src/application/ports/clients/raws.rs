use crate::domain::translation::{ChapterId, NovelId};
#[derive(Debug, thiserror::Error)]
pub enum RawsError {
    #[error("failed")]
    Failed,
}

pub trait RawsClient {
    async fn latest_chapter(&self, id: &NovelId) -> Result<ChapterId, RawsError>;
    async fn read_chapter(&self, id: &NovelId) -> Result<String, RawsError>;
}
