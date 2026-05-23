#[derive(Debug, thiserror::Error)]
pub enum TranslateError {
    #[error("connection  error: {0}")]
    ConnectionError(i16),
}

#[allow(async_fn_in_trait)]
pub trait TranslatorClient {
    async fn translate(&self, text: &str) -> Result<String, TranslateError>;
}
