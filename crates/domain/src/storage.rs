use pkg::macros::id_type;

id_type!(StorageBucket);
id_type!(StoragePath);
id_type!(StoragePrefix);

impl From<&str> for StoragePrefix {
    fn from(value: &str) -> Self {
        StoragePrefix(value.into())
    }
}

impl StoragePrefix {
    pub fn at(&self, other: StoragePrefix) -> StoragePrefix {
        if other.0.starts_with("/") {
            other
        } else if other.0.starts_with("./") {
            let s = format!("{}/{}", self.0, other.0.replacen("./", "", 1));
            StoragePrefix(s)
        } else {
            let s = format!("{}/{}", self.0, other.0);
            StoragePrefix(s)
        }
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
