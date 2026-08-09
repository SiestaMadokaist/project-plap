use aws_sdk_ec2::Client;

use crate::{
    application::ports::clients::compute::{ComputeEngine, ComputeEngines},
    domain::commands::compute::{ComputeInstanceID, ComputeRegion},
};

pub struct EC2 {
    region: ComputeRegion,
    client: Client,
}

pub struct EC2MultiRegion {
    regions: Vec<ComputeRegion>,
    client: Client,
}

impl EC2MultiRegion {
    pub fn new(regions: Vec<ComputeRegion>, client: Client) -> Self {
        EC2MultiRegion { regions, client }
    }
}

impl ComputeEngines for EC2MultiRegion {
    type Engine = EC2;
    fn get(&self, region: &ComputeRegion) -> Option<Self::Engine> {
        let region = self.regions.iter().find(|x| x == &region)?;
        let engine = EC2::new(region.clone(), self.client.clone());
        Some(engine)
    }
}

impl EC2 {
    pub fn new(region: ComputeRegion, client: Client) -> Self {
        Self { region, client }
    }
}

impl ComputeEngine for EC2 {
    fn region(&self) -> ComputeRegion {
        self.region.clone()
    }

    async fn stop(&self, _id: &ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }

    async fn launch(&self, _id: &ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }

    async fn terminate(&self, _id: &ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }

    async fn reboot(&self, _id: &ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }
}
