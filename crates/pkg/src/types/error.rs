use serde::Serialize;

use crate::displayable;

#[derive(Debug, Serialize, thiserror::Error)]
pub enum FormatError {
    InvalidPrefix(String),
    InvalidContent(String),
}
displayable!(FormatError);
