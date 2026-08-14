use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

pub trait WebUI {
    type Memo: Default;
}

pub(super) trait ExifTraits {
    fn positive(&self) -> Result<String, ExifError>;
    fn negative(&self) -> Result<String, ExifError>;
    fn checkpoint(&self) -> Result<String, ExifError>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExifError {
    InvalidRange,
    ParsingFailed,
    NotExtracted,
}
impl std::error::Error for ExifError {}

impl From<ExifError> for std::io::Error {
    fn from(value: ExifError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, value)
    }
}

displayable!(ExifError);
