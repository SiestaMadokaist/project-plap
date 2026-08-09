use serde::{Deserialize, Serialize};

use crate::{
    domain::{commands::compute::ComputeCommand, user::UserId},
    pkg::types::time::Second,
};
#[derive(Serialize, Deserialize)]
pub struct BillOptimization {
    idle_tolerance: Second,
    check_interval: Second,
    action: ComputeCommand,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchConfigCtx {
    LaunchConfigV2,
}

#[derive(Serialize, Deserialize)]
pub struct DiffusionConfigDomain {
    checkpoint: String,
    username: UserId,
    context: LaunchConfigCtx,
    region: String,
    bill_saving: BillOptimization,
}
