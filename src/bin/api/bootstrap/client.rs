use std::rc::Rc;

use aws_config::SdkConfig;
use rust_api::{
    application::ports::clients::container::{
        HasEngines, HasModelStorage, HasNotification, HasOutputStorage,
    },
    domain::commands::compute::ComputeRegion,
    infras::{
        compute::ec2::EC2MultiRegion, notifications::discord::Discord, storage::s3::S3Storage,
    },
};

use crate::env::ApiEnv;

// #[derive(Debug)]
pub struct ApiClients {
    notification: Discord,
    engines: EC2MultiRegion,
    output_storage: S3Storage,
    model_storage: S3Storage,
}

const MAX_SIZE: i64 = 100 * 1024 * 1024;

impl ApiClients {
    pub fn rc(env: &ApiEnv, config: &SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(env, config))
    }

    pub fn new(env: &ApiEnv, config: &SdkConfig) -> Self {
        let ec2sdk = aws_sdk_ec2::Client::new(&config);
        let regions = vec![ComputeRegion::AWSApSoutheast2, ComputeRegion::AWSUsEast1];
        let engines = EC2MultiRegion::new(regions, ec2sdk.clone());
        Self {
            notification: Discord::new(
                env.discord_username.clone(),
                env.discord_webhook_url.clone(),
            ),
            engines,
            output_storage: S3Storage::new(
                config.clone(),
                env.output_region
                    .as_str()
                    .try_into()
                    .expect("env.output_region must be a valid region"),
                env.output_bucket.clone(),
                env.output_prefix.clone(),
                "/tmp/".into(),
                MAX_SIZE,
            ),
            model_storage: S3Storage::new(
                config.clone(),
                env.model_region
                    .as_str()
                    .try_into()
                    .expect("env.model_region must be a valid region"),
                env.model_bucket.clone(),
                env.model_prefix.clone(),
                "/tmp/".into(),
                MAX_SIZE,
            ),
        }
    }
}

impl HasNotification for ApiClients {
    type Notification = Discord;
    fn notification(&self) -> &Self::Notification {
        &self.notification
    }
}

impl HasEngines for ApiClients {
    type Engines = EC2MultiRegion;
    fn engines(&self) -> &Self::Engines {
        &self.engines
    }
}

impl HasOutputStorage for ApiClients {
    type OutputStorage = S3Storage;
    fn output_storage(&self) -> &Self::OutputStorage {
        &self.output_storage
    }
}

impl HasModelStorage for ApiClients {
    type ModelStorage = S3Storage;
    fn model_storage(&self) -> &Self::ModelStorage {
        &self.model_storage
    }
}
