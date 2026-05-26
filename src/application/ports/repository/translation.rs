use crate::domain::{
    errors::DomainError,
    translation::{ChapterId, NovelId, RawSource, TranslationDomain},
};

#[allow(async_fn_in_trait)]
pub trait TranslationRepository {
    async fn latest(&self, id: &NovelId) -> Result<Option<TranslationDomain>, DomainError>;
    async fn insert(
        &self,
        prev: &TranslationDomain,
        chapter_id: &ChapterId,
    ) -> Result<TranslationDomain, DomainError>;

    async fn init(
        &self,
        id: &NovelId,
        chapter: &ChapterId,
        source: RawSource,
    ) -> Result<TranslationDomain, DomainError>;
}
