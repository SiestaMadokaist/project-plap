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
json_type!(DeleteTemplatePayload);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadTemplatePayload(pub StoryId);
json_type!(ReadTemplatePayload);

impl DTO for WriteTemplatePayload {}
impl DTO for DeleteTemplatePayload {}
impl DTO for ReadTemplatePayload {}
impl DTO for StoryId {}
impl DTO for StoragePath {}
impl DTO for Storyline {}
