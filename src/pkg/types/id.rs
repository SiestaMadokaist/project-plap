use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    S3,
    Civitai,
    Https,
}
#[derive(Serialize, Deserialize)]
pub struct ActionID(pub String);

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionStatus {
    InProgress,
    // Running is skipped
    // we use InProgress { progress: 1/n } to mark its Running
    Completed,
}
