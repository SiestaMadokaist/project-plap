use std::rc::Rc;

// use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use aws_config::SdkConfig;

use rust_api::{
    application::ports::clients::{
        compute_agent::ComputeAgent,
        container::{
            HasCivitai, HasComputeAgent, HasDiffusion, HasEngines, HasModelStorage,
            HasNotification, HasOutputStorage,
        },
        diffusions::DiffusionClient,
    },
    domain::commands::compute::ComputeRegion,
    infras::{
        civitai::client::CivitaiAPI,
        compute::ec2::EC2MultiRegion,
        compute_agent::{ec2::EC2Agent, localhost::LocalhostAgent},
        diffusions::a1111::A1111,
        notifications::discord::Discord,
        storage::s3::S3Storage,
    },
};

use crate::env::DiffusionEnv;

pub struct EC2DiffusionClients {
    output_storage: S3Storage,
    model_storage: S3Storage,
    notification: Discord,
    diffusion: Box<dyn DiffusionClient>,
    engines: EC2MultiRegion,
    agent: Box<dyn ComputeAgent>,
    civitai: CivitaiAPI,
}

impl EC2DiffusionClients {
    pub fn rc(env: &DiffusionEnv, config: SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(&env, config))
    }

    fn agent(env: &DiffusionEnv) -> Box<dyn ComputeAgent> {
        if env.localhost {
            Box::new(LocalhostAgent::new())
        } else {
            Box::new(EC2Agent::new())
        }
    }

    fn new(env: &DiffusionEnv, config: SdkConfig) -> Self {
        let regions: Vec<ComputeRegion> = vec![];
        let ec2sdk = aws_sdk_ec2::Client::new(&config);
        let engines = EC2MultiRegion::new(regions, ec2sdk.clone());
        let general_clients = Self {
            output_storage: S3Storage::new(
                config.clone(),
                env.output_region
                    .as_str()
                    .try_into()
                    .expect("env.output_region must be a valid region"),
                env.output_bucket.clone(),
                env.output_prefix.clone(),
                env.max_data_transfer,
            ),
            civitai: CivitaiAPI::new(
                env.civitai_host(),
                env.civitai_apikey.clone(),
                env.workdir.clone(),
            ),
            model_storage: S3Storage::new(
                config.clone(),
                env.model_region
                    .as_str()
                    .try_into()
                    .expect("env.model_region must be a valid region"),
                env.model_bucket.clone(),
                env.model_prefix.clone(),
                env.max_data_transfer,
            ),
            notification: Discord::new(env.discord_webhook_url.clone()),
            // TODO: pick A1111 vs ComfyUI at runtime (e.g. from Env), not wired yet
            diffusion: Box::new(A1111::new(String::new())),
            // TODO: not wired to Env yet, stub only
            engines,
            agent: Self::agent(env),
        };
        general_clients
    }
}

impl HasCivitai for EC2DiffusionClients {
    fn civitai(&self) -> &CivitaiAPI {
        &self.civitai
    }
}

impl HasNotification for EC2DiffusionClients {
    type Notification = Discord;
    fn notification(&self) -> &Self::Notification {
        &self.notification
    }
}
impl HasEngines for EC2DiffusionClients {
    type Engines = EC2MultiRegion;
    fn engines(&self) -> &Self::Engines {
        &self.engines
    }
}

impl HasDiffusion for EC2DiffusionClients {
    fn diffusion(&self) -> &dyn DiffusionClient {
        self.diffusion.as_ref()
    }
}

impl HasModelStorage for EC2DiffusionClients {
    type ModelStorage = S3Storage;
    fn model_storage(&self) -> &Self::ModelStorage {
        &self.model_storage
    }
}

impl HasOutputStorage for EC2DiffusionClients {
    type OutputStorage = S3Storage;
    fn output_storage(&self) -> &Self::OutputStorage {
        &self.output_storage
    }
}

impl HasComputeAgent for EC2DiffusionClients {
    fn agent(&self) -> &dyn ComputeAgent {
        self.agent.as_ref()
    }
}
