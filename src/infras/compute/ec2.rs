use aws_sdk_ec2::Client;

use crate::{
    application::ports::clients::compute::ComputeClient,
    domain::commands::compute::{ComputeInstanceID, ComputeRegion},
};

pub struct EC2Compute {
    region: ComputeRegion,
    client: Client,
}

impl EC2Compute {
    pub fn new(region: ComputeRegion, client: Client) -> Self {
        Self { region, client }
    }
}

impl ComputeClient for EC2Compute {
    fn region(&self) -> ComputeRegion {
        self.region.clone()
    }

    async fn stop(&self, _id: ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }

    async fn launch(&self, _id: ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }

    async fn terminate(&self, _id: ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }

    async fn reboot(&self, _id: ComputeInstanceID) -> anyhow::Result<()> {
        todo!()
    }
}
