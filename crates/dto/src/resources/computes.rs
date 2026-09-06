use domain::commands::compute::{ComputeArgs, ComputeInstance, ComputeRegion};
use pkg::json_type;
use serde::{Deserialize, Serialize};

use crate::response::DTO;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputeDTO(pub ComputeInstance);
impl DTO for ComputeDTO {}

/// Body of `POST /hq/instance/launch`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LaunchPayload {
    /// `true` launches a spot instance, `false` launches on-demand.
    pub spot: bool,
    /// Which of the caller's configured launch configs to use.
    pub region: ComputeRegion,
}
json_type!(LaunchPayload);

/// Body of `POST /hq/instance/control`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputeControlPayload(pub ComputeArgs);
json_type!(ComputeControlPayload);

/// Body of `POST /hq/instance/list`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputeListPayload {
    pub region: ComputeRegion,
}
json_type!(ComputeListPayload);

/// Response of `POST /hq/instance/list`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputeListResponse {
    pub instances: Vec<ComputeInstance>,
}
impl DTO for ComputeListResponse {}
