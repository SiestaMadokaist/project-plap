#[derive(Debug, thiserror::Error)]
pub enum TranslateError {
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("rate limited")]
    RateLimited,
    #[error("no translation")]
    EmptyResponse,
}

#[allow(async_fn_in_trait)]
pub trait TranslatorClient {
    async fn translate(&self, text: &str) -> Result<String, TranslateError>;
}
