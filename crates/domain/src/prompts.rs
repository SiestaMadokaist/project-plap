use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::{StorageBucket, StoragePath};
use pkg::macros::id_type;

id_type!(ImagePromptId);

/**
 * data structure to be stored in bigquery
 * mostly to be used as search engine of past generated prompt
 * for the full data / comfyui workflow just extract it from the image exif
 */
#[derive(Serialize, Deserialize)]
pub struct PromptHistory {
    /* positive clip used as prompt on text encoder */
    pub positive: String,
    /* negative clip used as prompt text encoder */
    pub negative: String,
    /* comma separated value of loras used on creating this image */
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

pub struct FuzzySearch {
    // comma separated string,
    tags: String,
}
