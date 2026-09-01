use domain::{
    storage::StoragePath,
    storyline::{StoryId, Storyline},
};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTemplatePayload(pub Storyline);
json_type!(WriteTemplatePayload);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTemplatePayload(pub StoryId);

impl DTO for WriteTemplatePayload {}
impl DTO for DeleteTemplatePayload {}
impl DTO for StoryId {}
impl DTO for StoragePath {}
