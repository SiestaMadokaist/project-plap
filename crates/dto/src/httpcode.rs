use domain::errors::DomainError;

pub(crate) fn code(e: &DomainError) -> u16 {
    match e {
        // --- client sent something wrong (4xx) ---
        DomainError::MissingPayload => 400,
        DomainError::NotFound => 404,
        DomainError::NotAllowed(_) => 403,
        DomainError::Prerequisite(_) => 412,
        DomainError::RateLimited => 429,

        // --- something failed on our side (5xx) ---
        DomainError::Unhandled => 500,
        DomainError::UnknownError(_) => 500,
        DomainError::Serialize(_) => 500,
        DomainError::NotImplemented => 501,
        DomainError::InvalidSelector(_) => 500,
        DomainError::TransferTooLarge { .. } => 500,
        DomainError::BillOptimization(_) => 500,

        // --- an upstream dependency failed (5xx) ---
        DomainError::ApiError(_) => 502,
        DomainError::InvalidRegion(_) => 502,
        DomainError::HttpError(_) => 502,
        DomainError::EmptyResponse => 502,
        DomainError::MissingContent => 502,
        DomainError::Disconnected(_) => 503,
        DomainError::HttpConnectionFailed(_) => 504,
    }
}
