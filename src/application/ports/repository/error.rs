#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("failed to connect to {0}")]
    Disconnected(String),
    #[error("serialization failure: {0}")]
    Serialize(String),
    #[error("serialization failure: {0}")]
    Database(String),
}
