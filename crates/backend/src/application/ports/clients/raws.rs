use domain::{
    errors::DomainError,
    translation::{ChapterId, NovelId},
};

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait RawsClient {
    async fn latest(&self, novel_id: &NovelId) -> Result<ChapterId, DomainError>;
    async fn read(&self, novel_id: &NovelId, chapter_id: &ChapterId)
        -> Result<String, DomainError>;
}
