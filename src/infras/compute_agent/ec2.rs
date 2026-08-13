use std::cell::OnceCell;

use serde::{Deserialize, Serialize};

use crate::{
    application::ports::clients::compute_agent::ComputeAgent,
    domain::commands::compute::{ComputeInstanceID, ComputeRegion},
};

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
}
pub struct EC2Agent {
    memo: Memo,
}

impl EC2Agent {
    pub fn new() -> Self {
        Self {
            memo: Memo {
                document: OnceCell::new(),
            },
        }
    }
    async fn document(&self) -> anyhow::Result<Document> {
        //  const { data: token } = await axios.put<string>(`${METADATA}/api/token`, null, {
        //     headers: { "X-aws-ec2-metadata-token-ttl-seconds": "21600" },
        // });
        // const { data: region } = await axios.get<string>(`${METADATA}/dynamic/instance-identity/document`, {
        //     headers: { "X-aws-ec2-metadata-token": token },
        // });
        let memoized = self.memo.document.get_or_init(|| todo!());
        Ok(memoized.clone())
    }
}

#[async_trait::async_trait(?Send)]
impl ComputeAgent for EC2Agent {
    async fn ip(&self) -> anyhow::Result<String> {
        todo!();
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
