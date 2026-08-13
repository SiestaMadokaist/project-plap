use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::{
    application::ports::clients::compute_agent::ComputeAgent,
    domain::commands::compute::{ComputeInstanceID, ComputeRegion},
};

const METADATA: &str = "http://169.254.169.254/latest";

// pub trait ComputeEngine<Client> {
//     // {
//     //   accountId: '674152176016',
//     //   architecture: 'x86_64',
//     //   availabilityZone: 'us-east-1d',
//     //   billingProducts: null,
//     //   devpayProductCodes: null,
//     //   marketplaceProductCodes: null,
//     //   imageId: 'ami-02d9713e6a0f5121b',
//     //   instanceId: 'i-00a202d44865a93f4',
//     //   instanceType: 'g6.xlarge',
//     //   kernelId: null,
//     //   pendingTime: '2026-08-13T02:02:00Z',
//     //   privateIp: '172.31.7.6',
//     //   ramdiskId: null,
//     //   region: 'us-east-1',
//     //   version: '2017-09-30'
//     // }
//     fn document(&self) -> anyhow::Result<()>; // todo
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    region: ComputeRegion,
    #[serde(rename = "instanceId")]
    instance_id: ComputeInstanceID,
}
struct Memo {
    document: OnceCell<Document>,
    ip: OnceCell<String>,
}
pub struct EC2Agent {
    client: reqwest::Client,
    memo: Memo,
}

impl EC2Agent {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            memo: Memo {
                document: OnceCell::new(),
                ip: OnceCell::new(),
            },
        }
    }

    async fn token(&self) -> anyhow::Result<String> {
        let token = self
            .client
            .put(format!("{METADATA}/api/token"))
            .header("X-aws-ec2-metadata-token-ttl-seconds", "21600")
            .send()
            .await?
            .text()
            .await?;
        Ok(token)
    }

    async fn document(&self) -> anyhow::Result<Document> {
        let memoized = self
            .memo
            .document
            .get_or_try_init(async || {
                let token = self.token().await?;

                self.client
                    .get(format!("{METADATA}/dynamic/instance-identity/document"))
                    .header("X-aws-ec2-metadata-token", token)
                    .send()
                    .await?
                    .json::<Document>()
                    .await
                    .map_err(anyhow::Error::from)
            })
            .await?;
        Ok(memoized.clone())
    }
}

#[async_trait::async_trait(?Send)]
impl ComputeAgent for EC2Agent {
    async fn ip(&self) -> anyhow::Result<String> {
        let memoized = self
            .memo
            .ip
            .get_or_try_init(async || {
                let token = self.token().await?;

                self.client
                    .get(format!("{METADATA}/meta-data/public-ipv4"))
                    .header("X-aws-ec2-metadata-token", token)
                    .send()
                    .await?
                    .text()
                    .await
                    .map_err(anyhow::Error::from)
            })
            .await?;
        Ok(memoized.clone())
    }
    async fn instance_id(&self) -> anyhow::Result<ComputeInstanceID> {
        let doc = self.document().await?;
        Ok(doc.instance_id)
    }
    async fn region(&self) -> anyhow::Result<ComputeRegion> {
        let doc = self.document().await?;
        Ok(doc.region)
    }
}
