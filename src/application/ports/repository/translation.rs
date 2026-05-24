use aws_sdk_dynamodb::Error;

use crate::domain::translation::{ChapterId, NovelId, Translation};

pub trait TranslationRepository {
    async fn latest(&self, id: &NovelId) -> Result<Option<Translation>, Error>;
    async fn set_latest(&self, id: &NovelId, chapter: &ChapterId) -> Result<Translation, Error>;
}
