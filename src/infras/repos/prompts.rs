use crate::{
    application::ports::repository::prompt_history::{PromptHistoryError, PromptHistoryRepository},
    domain::{prompts::ImagePromptDomain, storage::StoragePath},
    pkg::types::strings::CommaSeparated,
};

pub struct PromptRepository {}

impl PromptRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl PromptHistoryRepository for PromptRepository {
    #[cfg(feature = "future")]
    async fn fuzzy_search(
        &self,
        tags: &CommaSeparated,
    ) -> Result<Vec<StoragePath>, PromptHistoryError> {
        todo!();
    }

    async fn insert(&self, row: ImagePromptDomain) -> Result<(), PromptHistoryError> {
        todo!();
    }
}
