use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

#[derive(Clone, Serialize, Deserialize)]
pub struct CommaSeparated(pub String);

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
