use crate::domain::errors::DomainError;

#[allow(async_fn_in_trait)]
pub trait TranslatorClient {
    async fn translate(&self, text: &str) -> Result<String, DomainError>;
}
