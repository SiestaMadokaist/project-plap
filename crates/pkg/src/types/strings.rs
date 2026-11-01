use serde::{Deserialize, Serialize};

use crate::{macros::displayable, types::error::FormatError};
#[derive(Clone, Serialize, Deserialize)]
pub struct CommaSeparated(pub String);

pub trait StringFormat {
    fn validate(s: &str) -> Result<(), FormatError>;
    fn as_str(&self) -> &str;
}

impl CommaSeparated {
    pub fn split(&self) -> Vec<&str> {
        let s: Vec<&str> = self.0.split(", ").collect();
        s
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct URL(pub String);

impl URL {
    pub fn e(&self, e: &str) -> Self {
        let s = format!("{}{}", self.0, e);
        Self(s)
    }
}
displayable!(URL);
