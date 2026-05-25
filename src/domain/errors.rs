// use ;

use crate::application::ports::clients::{raws::RawsError, translator::TranslateError};

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("failed")]
    Unhandled,
}

impl From<aws_sdk_dynamodb::Error> for DomainError {
    fn from(_value: aws_sdk_dynamodb::Error) -> Self {
        return DomainError::Unhandled;
    }
}

impl From<RawsError> for DomainError {
    fn from(_value: RawsError) -> Self {
        return DomainError::Unhandled;
    }
}

impl From<TranslateError> for DomainError {
    fn from(_value: TranslateError) -> Self {
        return DomainError::Unhandled;
    }
}
