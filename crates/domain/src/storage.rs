use pkg::macros::id_type;
use serde::{Deserialize, Serialize};

id_type!(StorageBucket);
id_type!(StoragePath);
id_type!(StoragePrefix);

impl From<&str> for StoragePrefix {
    fn from(value: &str) -> Self {
        StoragePrefix(value.into())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirTree {
    pub paths: Vec<StoragePath>,
    pub prefixes: Vec<StoragePrefix>,
}

impl StoragePrefix {
    pub fn add(&self, other: &StoragePath) -> StoragePath {
        let s = format!("{}{}", self.0, other.0);
        StoragePath(s)
    }

    pub fn at(&self, other: &StoragePrefix) -> StoragePrefix {
        let s = format!("{}{}", self.0, other.0);
        StoragePrefix(s)
    }
}

#[cfg(feature = "future")]
#[derive(Serialize, Deserialize)]
pub struct ItemVersion {
    pub key: Option<String>,
    pub version_id: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub size: Option<i64>,
    pub e_tag: Option<String>,
}
