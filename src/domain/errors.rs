#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("failed")]
    Unhandled,

    #[error("prerequisite not satisfied")]
    Prerequisite(String),

    #[error("not implemented")]
    NotImplemented,

    #[error("disconnected error: {0}")]
    Disconnected(String),

    #[error("serialization error: {0}")]
    Serialize(String),

    #[error("rate limited by upstream API")]
    RateLimited,

    #[error("upstream API error: {0}")]
    ApiError(String),

    #[error("upstream returned an empty response")]
    EmptyResponse,

    #[error("upstream response had no content")]
    MissingContent,

    #[error("http connection failed: {0}")]
    HttpConnectionFailed(String),

    #[error("http error: {0}")]
    HttpError(String),

    #[error("invalid selector: {0}")]
    InvalidSelector(String),

    #[error("Bill Optimization: {0}")]
    BillOptimization(String),

    #[error("transfer size {size} bytes exceeds the {limit} byte limit")]
    TransferTooLarge { size: u64, limit: u64 },
}
