use serde::{Deserialize, Serialize};

use crate::pkg::macros::displayable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionId(pub u32);
displayable!(VersionId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelId(pub u32);
displayable!(ModelId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo();
