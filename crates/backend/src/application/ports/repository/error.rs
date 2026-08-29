use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum RepositoryError<Id: fmt::Debug + fmt::Display> {
    #[error("{0} not found")]
    NotFound(Id),
    #[error("failed to connect to {0}")]
    Disconnected(String),
    #[error("serialization failure: {0}")]
    Serialize(String),
    #[error("serialization failure: {0}")]
    Database(String),
    #[error("conflict: {0}")]
    Conflict(String),
}
