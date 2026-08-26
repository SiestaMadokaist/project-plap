use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum DomainError {
    #[error("failed")]
    Unhandled,

    #[error("unknown error: {0}")]
    UnknownError(String),

    #[error("prerequisite not satisfied")]
    Prerequisite(String),

    #[error("action: {0} is not allowed")]
    NotAllowed(String),

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

    #[error("payload incomplete")]
    MissingPayload,

    #[error("http connection failed: {0}")]
    HttpConnectionFailed(String),

    #[error("http error: {0}")]
    HttpError(String),

    #[error("invalid selector: {0}")]
    InvalidSelector(String),

    #[error("Invalid region: {0}")]
    InvalidRegion(String),

    #[error("Bill Optimization: {0}")]
    BillOptimization(String),

    #[error("transfer size {size} bytes exceeds the {limit} byte limit")]
    TransferTooLarge { size: u64, limit: u64 },
}

impl From<serde_json::Error> for DomainError {
    fn from(value: serde_json::Error) -> Self {
        DomainError::Serialize(value.to_string())
    }
}
