use crate::{
    application::ports::repository::translation::TranslationRepository,
    domain::{
        errors::DomainError,
        translation::{
            ChapterId, NovelId, RawSource, Status, TranslationDomain, TranslationGetter,
        },
        user::{Email, User, UserId},
    },
    pkg::types::time::Timestamp,
};
use aws_sdk_dynamodb::Client;
use serde::{Deserialize, Serialize};

use super::table::{table_name, user_pk, USER_SK};

// Internal DynamoDB representation — PK/SK envelope wrapping domain fields.
// Kept private: callers only see User and UserRepoError.
#[derive(Debug, Serialize, Deserialize)]
struct TranslationDDB {
    novel_id: NovelId,
    chapter_id: ChapterId,
    source: RawSource,
    title: String,
    created_at: Timestamp,
    updated_at: Timestamp,
    status: Status,
}

impl From<&TranslationDomain> for TranslationDDB {
    fn from(tl: &TranslationDomain) -> Self {
        Self {
            novel_id: tl.novel_id().clone(),
            chapter_id: *tl.chapter_id(),
            source: *tl.source(),
            title: tl.title().clone(),
            created_at: *tl.created_at(),
            updated_at: *tl.updated_at(),
            status: tl.status().clone(),
        }
    }
}

impl TranslationGetter for TranslationDDB {
    fn title(&self) -> &String {
        return &self.title;
    }

    fn status(&self) -> &Status {
        return &self.status;
    }

    fn created_at(&self) -> &Timestamp {
        return &self.created_at;
    }

    fn updated_at(&self) -> &Timestamp {
        return &self.updated_at;
    }

    fn chapter_id(&self) -> &ChapterId {
        return &self.chapter_id;
    }

    fn novel_id(&self) -> &NovelId {
        return &self.novel_id;
    }

    fn source(&self) -> &RawSource {
        return &self.source;
    }
}

pub struct DDBTranslationRepository {
    client: Client,
    table: String,
}

impl DDBTranslationRepository {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            table: table_name(),
        }
    }
}

impl TranslationRepository for DDBTranslationRepository {
    async fn latest(&self, id: &NovelId) -> Result<Option<TranslationDomain>, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn init(
        &self,
        id: &NovelId,
        chapter: &ChapterId,
        source: RawSource,
    ) -> Result<TranslationDomain, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn insert(
        &self,
        novel_id: &NovelId,
        chapter_id: &ChapterId,
    ) -> Result<TranslationDomain, DomainError> {
        Err(DomainError::NotImplemented)
    }
}
