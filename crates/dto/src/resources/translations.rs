use domain::translation::{ChapterId, NovelId};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitPayload {
    pub novel_id: NovelId,
    pub starting_chapter: Option<ChapterId>,
    pub title: String,
}
json_type!(InitPayload);
impl DTO for InitPayload {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RunPayload {
    pub novel_id: NovelId,
}
