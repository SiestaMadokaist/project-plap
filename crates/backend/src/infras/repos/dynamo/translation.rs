use crate::application::ports::repository::translation::TranslationRepository;
use domain::{
    errors::DomainError,
    translation::{ChapterId, NovelId, RawSource, Status, TranslationDomain, TranslationGetter},
};
use pkg::types::time::Timestamp;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use serde::{Deserialize, Serialize};
use serde_dynamo::{from_item, to_item};

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
            status: tl.status(),
        }
    }
}

impl TranslationGetter for TranslationDDB {
    fn title(&self) -> &String {
        &self.title
    }

    fn status(&self) -> Status {
        self.status
    }

    fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    fn chapter_id(&self) -> &ChapterId {
        &self.chapter_id
    }

    fn novel_id(&self) -> &NovelId {
        &self.novel_id
    }

    fn source(&self) -> &RawSource {
        &self.source
    }
}

pub struct DDBTranslationRepository {
    client: Client,
    table: String,
}

impl DDBTranslationRepository {
    pub fn new(client: Client, table: String) -> Self {
        Self { client, table }
    }

    async fn put(&self, domain: &TranslationDomain) -> Result<(), DomainError> {
        let av_map = to_item(TranslationDDB::from(domain))
            .map_err(|e| DomainError::Serialize(e.to_string()))?;
        tracing::info!(item = self.table, "tablename");
        tracing::info!(item = ?av_map, "putting item to dynamo");
        self.client
            .put_item()
            .table_name(&self.table)
            .set_item(Some(av_map))
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, debug = ?e, "put_item failed");
                DomainError::Disconnected(e.to_string())
            })?;

        Ok(())
    }
}

impl TranslationRepository for DDBTranslationRepository {
    async fn latest(&self, id: &NovelId) -> Result<Option<TranslationDomain>, DomainError> {
        let out = self
            .client
            .query()
            .table_name(&self.table)
            .key_condition_expression("novel_id = :novel_id")
            .expression_attribute_values(":novel_id", AttributeValue::S(id.0.clone()))
            .scan_index_forward(false)
            .limit(1)
            .send()
            .await
            .map_err(|e| DomainError::Disconnected(e.to_string()))?;

        let item = match out.items.and_then(|mut v| v.drain(..).next()) {
            None => return Ok(None),
            Some(item) => item,
        };

        let ddb: TranslationDDB =
            from_item(item).map_err(|e| DomainError::Serialize(e.to_string()))?;

        Ok(Some(ddb.into()))
    }

    async fn init(
        &self,
        id: &NovelId,
        chapter: &ChapterId,
        title: &str,
        source: RawSource,
    ) -> Result<TranslationDomain, DomainError> {
        let domain = TranslationDomain::new(id.clone(), *chapter, title, Some(source));
        self.put(&domain).await?;
        Ok(domain)
    }

    async fn insert(
        &self,
        prev: &TranslationDomain,
        chapter_id: &ChapterId,
    ) -> Result<TranslationDomain, DomainError> {
        let domain = TranslationDomain::new(
            prev.novel_id().clone(),
            *chapter_id,
            prev.title(),
            Some(*prev.source()),
        );
        self.put(&domain).await?;
        Ok(domain)
    }
}
