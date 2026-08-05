use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::pkg::macros::id_type;

id_type!(StorageBucket);
id_type!(StoragePath);
id_type!(StoragePrefix);

#[derive(Serialize, Deserialize)]
pub struct ItemVersion {
    pub key: Option<String>,
    pub version_id: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub size: Option<i64>,
    pub e_tag: Option<String>,
}
