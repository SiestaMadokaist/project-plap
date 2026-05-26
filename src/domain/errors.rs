// use ;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("failed")]
    Unhandled,

    #[error("not implemented")]
    NotImplemented,
}

impl From<aws_sdk_dynamodb::Error> for DomainError {
    fn from(_value: aws_sdk_dynamodb::Error) -> Self {
        return DomainError::Unhandled;
    }
}
