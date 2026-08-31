use std::env::VarError;

use serde::{Deserialize, Serialize};

use pkg::macros::{displayable, id_type};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ComputeRegion {
    #[serde(rename = "ap-southeast-1")]
    AwsApSoutheast1,
    #[serde(rename = "ap-southeast-2")]
    AWSApSoutheast2,
    #[serde(rename = "ap-southeast-3")]
    AWSApSoutheast3,
    #[serde(rename = "us-east-1")]
    AWSUsEast1,
}
displayable!(ComputeRegion);

impl ComputeRegion {
    pub fn from_env(s: String) -> Result<ComputeRegion, VarError> {
        let invalid_msg = format!("invalid region: {}", s);
        ComputeRegion::try_from(s.as_str()).map_err(|_| VarError::NotUnicode(invalid_msg.into()))
    }
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum ComputeError {
    #[error("Region {0} is not configured")]
    InvalidRegion(String),
}

impl TryFrom<&str> for ComputeRegion {
    type Error = ComputeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ap-southeast-1" => Ok(ComputeRegion::AwsApSoutheast1),
            "ap-southeast-2" => Ok(ComputeRegion::AWSApSoutheast2),
            "ap-southeast-3" => Ok(ComputeRegion::AWSApSoutheast3),
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComputeCommand {
    Terminate,
    Stop,
    Reboot,
}

#[cfg(test)]
mod tests {
    use crate::commands::compute::{ComputeArgs, ComputeCommand, ComputeRegion};

    #[test]
    fn shape_test() -> anyhow::Result<()> {
        let buffer = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../samples/inputs/jsons/domain/commands/compute.json"
        ))?;
        let command: ComputeArgs = serde_json::from_slice(&buffer)?;
        assert_eq!(command.command, ComputeCommand::Terminate);
        assert_eq!(command.region, ComputeRegion::AWSUsEast1);
        assert_eq!(command.instance_id.0, "test123");
        Ok(())
    }
}
