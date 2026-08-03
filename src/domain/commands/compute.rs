use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeArgs {
    instance_id: String,
    command: ComputeCommand,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeCommand {
    Terminate,
    Stop,
    Reboot,
}
