use serde::{Deserialize, Serialize};

/// Placeholder DTO so the crate compiles. Replace with real request/response
/// types as endpoints are ported over.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
}
