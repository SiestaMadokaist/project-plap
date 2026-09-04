use serde::{Deserialize, Serialize};

use crate::commands::compute::{ComputeCommand, LaunchConfig};
use pkg::{auth::claims::Username, types::time::Second};
#[derive(Clone, Serialize, Deserialize, Copy)]
pub struct BillOptimization {
    pub idle_tolerance: Second,
    pub check_interval: Second,
    pub action: ComputeCommand,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HotreloadDomain {
    username: Username,
    #[serde(flatten)]
    config: HotReloadCfg,
}

impl HotreloadDomain {
    pub fn new(username: Username, config: HotReloadCfg) -> Self {
        Self { username, config }
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn config(&self) -> &HotReloadCfg {
        &self.config
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "context", content = "data")]
pub enum HotReloadCfg {
    #[serde(rename = "bill")]
    Bill(BillOptimization),
    /// One entry per region the user has configured a launch template for -
    /// `HotReloadRepository::launch_config` filters this in memory by the
    /// requested region (DynamoDB can't query inside a nested list attribute).
    #[serde(rename = "launch")]
    Launch(Vec<LaunchConfig>),
}
