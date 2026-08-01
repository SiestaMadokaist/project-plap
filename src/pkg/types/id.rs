use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    S3,
    Civitai,
    Https,
}
pub struct ActionID(pub String);
