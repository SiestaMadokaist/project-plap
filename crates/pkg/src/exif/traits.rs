use serde::{Deserialize, Serialize};

use crate::macros::displayable;

pub trait WebUI {
    type Memo: Default;
}

pub trait ExifTraits {
    fn positive(&self) -> Result<&str, ExifError>;
    fn negative(&self) -> Result<&str, ExifError>;
    fn checkpoint(&self) -> Result<&str, ExifError>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExifError {
    InvalidRange,
    ParsingFailed,
    NotExtracted,
}
impl std::error::Error for ExifError {}

pub struct Exif<S: WebUI> {
    pub(super) data: Vec<u8>,
    pub(super) memo: S::Memo,
    pub(super) _src: std::marker::PhantomData<S>,
}

impl From<ExifError> for std::io::Error {
    fn from(value: ExifError) -> Self {
        std::io::Error::other(value)
    }
}

displayable!(ExifError);
