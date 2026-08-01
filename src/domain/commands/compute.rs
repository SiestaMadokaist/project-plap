use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ComputeArgs {
    instance_id: String,
    command: ComputeCommand,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeCommand {
    Terminate,
    Stop,
    Reboot,
}
