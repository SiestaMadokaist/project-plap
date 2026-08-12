use serde::{Deserialize, Serialize};

use crate::pkg::macros::{displayable, id_type};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ComputeRegion {
    #[serde(rename = "ap-southeast-2")]
    AWSApSoutheast2,
    #[serde(rename = "us-east-1")]
    AWSUsEast1,
}
displayable!(ComputeRegion);

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum ComputeError {
    #[error("Region {0} is not configured")]
    InvalidRegion(String),
}

impl TryFrom<&str> for ComputeRegion {
    type Error = ComputeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ap-southeast-2" => Ok(ComputeRegion::AWSApSoutheast2),
            "us-east-1" => Ok(ComputeRegion::AWSUsEast1),
            other => Err(ComputeError::InvalidRegion(other.to_string())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeArgs {
    pub instance_id: ComputeInstanceID,
    pub command: ComputeCommand,
    pub region: ComputeRegion,
}
id_type!(ComputeInstanceID);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ComputeCommand {
    Terminate,
    Stop,
    Reboot,
}
