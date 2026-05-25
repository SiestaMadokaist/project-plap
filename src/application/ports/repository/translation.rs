use aws_sdk_dynamodb::Error;

use crate::domain::translation::{ChapterId, NovelId, TranslationProgress};

pub trait TranslationRepository {
    async fn latest(&self, id: &NovelId) -> Result<Option<TranslationProgress>, Error>;
    async fn set_latest(
        &self,
        id: &NovelId,
        chapter: &ChapterId,
    ) -> Result<TranslationProgress, Error>;
}
