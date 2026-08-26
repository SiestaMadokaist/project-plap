use domain::errors::DomainError;

// `DomainError` lives in the separate `domain` crate, so `impl From<reqwest::Error> for
// DomainError` would violate the orphan rules here. Shared via `.map_err(reqwest_error)`
// at call sites instead of a blanket `From` impl.
pub(crate) fn reqwest_error(e: reqwest::Error) -> DomainError {
    if e.is_connect() || e.is_timeout() {
        DomainError::HttpConnectionFailed(e.to_string())
    } else {
        DomainError::HttpError(e.to_string())
    }
}
