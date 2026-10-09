use aws_config::SdkConfig;
use domain::{
    commands::compute::{ComputeInstance, ComputeInstanceID, ComputeRegion, LaunchConfig},
    errors::DomainError,
};
use pkg::auth::claims::Username;

use crate::application::ports::clients::compute::{ComputeEngine, ComputeEngines};

/// Stand-in for `EC2MultiRegion` when built without `remote-ec2`: `get` never
/// yields an engine, so every compute usecase reports the region as unavailable.
#[derive(Debug)]
pub struct NoEngines;

impl NoEngines {
    /// Takes the config only to match `EC2MultiRegion::new`.
    pub fn new(_config: SdkConfig) -> Self {
        NoEngines
    }
}

/// Uninhabited: since `NoEngines::get` always returns `None`, no value of this
/// type can exist, so the methods below are provably unreachable.
pub enum NoEngine {}

impl ComputeEngines for NoEngines {
    type Engine = NoEngine;
    fn get(&self, _region: &ComputeRegion) -> Option<Self::Engine> {
        None
    }
}

impl ComputeEngine for NoEngine {
    async fn stop(&self, _id: &ComputeInstanceID) -> Result<(), DomainError> {
        match *self {}
    }
    async fn start(&self, _id: &ComputeInstanceID) -> Result<(), DomainError> {
        match *self {}
    }
    async fn terminate(&self, _id: &ComputeInstanceID) -> Result<(), DomainError> {
        match *self {}
    }
    async fn reboot(&self, _id: &ComputeInstanceID) -> Result<(), DomainError> {
        match *self {}
    }
    async fn launch(
        &self,
        _config: &LaunchConfig,
        _username: &Username,
        _script: &Option<String>,
        _spot: bool,
    ) -> Result<ComputeInstance, DomainError> {
        match *self {}
    }
    async fn list(&self) -> Result<Vec<ComputeInstance>, DomainError> {
        match *self {}
    }
    async fn open(&self, _ip: &str) -> Result<(), DomainError> {
        match *self {}
    }
    fn region(&self) -> ComputeRegion {
        match *self {}
    }
}
