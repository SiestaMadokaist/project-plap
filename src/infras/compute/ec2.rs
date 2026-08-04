use aws_sdk_ec2::Client;

use crate::application::ports::clients::compute::ComputeClient;

pub struct EC2Compute {
    client: Client,
    instance_id: String,
}

impl EC2Compute {
    pub fn new(client: Client, instance_id: String) -> Self {
        Self {
            client,
            instance_id,
        }
    }
}

impl ComputeClient for EC2Compute {
    async fn stop(&self) -> anyhow::Result<()> {
        let _ = (&self.client, &self.instance_id);
        todo!()
    }

    async fn launch(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn terminate(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn reboot(&self) -> anyhow::Result<()> {
        todo!()
    }
}
