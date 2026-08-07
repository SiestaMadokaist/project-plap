use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::pkg::macros::id_type;

#[derive(Debug, Clone, Copy)]
pub enum ComputeRegion {
    ApSoutheast2,
    UsEast1,
}

impl ComputeRegion {
    fn as_str(&self) -> &'static str {
        match self {
            ComputeRegion::ApSoutheast2 => "ap-southeast-2",
            ComputeRegion::UsEast1 => "us-east-1",
        }
    }
}

impl Serialize for ComputeRegion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ComputeRegion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "ap-southeast-2" => Ok(ComputeRegion::ApSoutheast2),
            "us-east-1" => Ok(ComputeRegion::UsEast1),
            _ => Err(D::Error::custom(format!("unknown compute region: {s}"))),
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
