use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::pkg::types::unit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowId(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceConfig {
    workflow_id: Option<WorkflowId>,
    positive: String,
    negative: String,
    width: unit::Px,
    height: unit::Px,
    steps: unit::Index1,
    /// how many image generated per request
    #[serde(deserialize_with = "deserialize_n_iter")]
    n_iter: unit::Index1,
    seed: u32,
}

fn deserialize_n_iter<'de, D>(deserializer: D) -> Result<unit::Index1, D::Error>
where
    D: Deserializer<'de>,
{
    let n_iter = unit::Index1::deserialize(deserializer)?;
    if n_iter.0 >= 5 {
        return Err(D::Error::custom("n_iter must be less than 5"));
    }
    Ok(n_iter)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceArgs {
    pub config: InferenceConfig,
}
