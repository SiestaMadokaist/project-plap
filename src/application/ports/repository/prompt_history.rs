use crate::{
    application::ports::repository::error::RepositoryError,
    domain::{prompts::ImagePromptDomain, storage::StoragePath},
    pkg::types::strings::CommaSeparated,
};

pub type PromptHistoryError = RepositoryError<StoragePath>;
#[allow(async_fn_in_trait)]
pub trait PromptHistoryRepository {
    async fn insert(&self, row: ImagePromptDomain) -> Result<(), PromptHistoryError>;
    /** @todo
       const queryTemplate = (tableId: string, tags: string[], minimum: number) => `WITH scored AS (
       SELECT *,
           (
           SELECT COUNT(*)
           FROM UNNEST([${tags.map((tag) => `'${tag}'`).join(', ')}]) AS keyword
           WHERE LOWER(prompts) LIKE CONCAT('%', keyword, '%')
           ) AS match_count
       FROM  ${tableId}
       )
       SELECT path, bucket, created_at
       FROM scored
       WHERE match_count >= ${minimum}
       ORDER BY match_count DESC`;
    */
    async fn fuzzy_search(
        &self,
        tags: &CommaSeparated,
    ) -> Result<Vec<StoragePath>, PromptHistoryError>;
}
