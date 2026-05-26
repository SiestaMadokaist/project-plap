use crate::domain::errors::DomainError;

impl From<aws_sdk_dynamodb::Error> for DomainError {
    fn from(value: aws_sdk_dynamodb::Error) -> Self {
        DomainError::Disconnected(value.to_string())
    }
}
