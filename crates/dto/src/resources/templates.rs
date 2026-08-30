use domain::{
    storage::StoragePath,
    storyline::{StoryId, Storyline},
};
use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTemplatePayload(pub Storyline);
impl DTO for WriteTemplatePayload {}
impl DTO for StoryId {}
impl DTO for StoragePath {}
