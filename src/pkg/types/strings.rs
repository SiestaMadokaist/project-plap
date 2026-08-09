use serde::{Deserialize, Serialize};

type Tags = CommaSeparated;
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
