// use ;

use crate::application::ports::clients::raws::RawsError;

#[derive(Debug, thiserror::Error)]
pub enum UsecaseError {
    #[error("failed")]
    Failed,
}

impl From<aws_sdk_dynamodb::Error> for UsecaseError {
    fn from(_value: aws_sdk_dynamodb::Error) -> Self {
        return UsecaseError::Failed;
    }
}

impl From<RawsError> for UsecaseError {
    fn from(_value: RawsError) -> Self {
        return UsecaseError::Failed;
    }
}
