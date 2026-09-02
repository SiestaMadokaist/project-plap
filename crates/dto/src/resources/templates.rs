use domain::{
    storage::StoragePath,
    storyline::{StoryTemplateId, Storyline},
};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::{resources::list::ListResponse, response::DTO};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTemplatePayload(pub Storyline);
json_type!(WriteTemplatePayload);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTemplatePayload(pub StoryTemplateId);
json_type!(DeleteTemplatePayload);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadTemplatePayload(pub StoryTemplateId);
json_type!(ReadTemplatePayload);

pub type ListStoryTemplate = ListResponse<StoryTemplateId>;

impl DTO for ListStoryTemplate {}
impl DTO for WriteTemplatePayload {}
impl DTO for DeleteTemplatePayload {}
impl DTO for ReadTemplatePayload {}
impl DTO for StoryTemplateId {}
impl DTO for StoragePath {}
impl DTO for Storyline {}
