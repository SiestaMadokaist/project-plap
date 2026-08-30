use serde::{Deserialize, Serialize};

use crate::commands::compute::ComputeCommand;
use pkg::{auth::claims::Username, types::time::Second};
#[derive(Serialize, Deserialize)]
pub struct BillOptimization {
    idle_tolerance: Second,
    check_interval: Second,
    action: ComputeCommand,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HotReloadService {
    EC2Diffusion,
}

#[derive(Serialize, Deserialize)]
pub struct DiffusionConfigDomain {
    checkpoint: String,
    username: Username,
    svc: HotReloadService,
    region: String,
    bill_saving: BillOptimization,
}
