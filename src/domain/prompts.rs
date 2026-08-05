use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::storage::{StorageBucket, StoragePath},
    pkg::macros::id_type,
};

id_type!(ImagePromptId);

#[derive(Serialize, Deserialize)]
pub struct ImagePromptDomain {
    pub prompts: String,
    pub negatives: String,
    pub loras: String,
    pub checkpoint: String,
    pub bucket: StorageBucket,
    pub path: StoragePath,
    /** image creation datetime */
    pub created_at: DateTime<Utc>,
    /**
     * data row recording datetime
     * legacy from daily batch mechanism
     */
    pub recorded_at: DateTime<Utc>,
}
