use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeArgs {
    pub instance_id: String,
    pub command: ComputeCommand,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeCommand {
    Terminate,
    Stop,
    Reboot,
}
