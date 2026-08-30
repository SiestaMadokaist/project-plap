use std::fmt;

use domain::errors::DomainError;
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

impl<T: fmt::Debug + fmt::Display> From<RepositoryError<T>> for DomainError {
    fn from(value: RepositoryError<T>) -> Self {
        let e = match &value {
            RepositoryError::Conflict(x) => DomainError::Prerequisite(x.into()),
            RepositoryError::Database(x) => DomainError::Disconnected(x.into()),
            RepositoryError::Disconnected(x) => DomainError::Disconnected(x.into()),
            RepositoryError::NotFound(_) => DomainError::NotFound,
            RepositoryError::Serialize(x) => DomainError::Serialize(x.into()), // _ => DomainError::EmptyResponse,
        };
        tracing::info!("repository error: {} were thrown as {}", &value, &e);
        e
    }
}
