use std::rc::Rc;

// use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use aws_config::SdkConfig;

use crate::{
    application::ports::clients::{
        compute_agent::ComputeAgent,
        container::{HasComputeAgent, HasDiffusion, HasEngines, HasNotification, HasStorage},
        diffusions::DiffusionClient,
    },
    config::diffusion_env::DiffusionEnv,
    domain::commands::compute::ComputeRegion,
    infras::{
        compute::ec2::EC2MultiRegion,
        compute_agent::{ec2::EC2Agent, localhost::LocalhostAgent},
        diffusions::a1111::A1111,
        notifications::discord::Discord,
        storage::s3::S3Storage,
    },
};

pub struct EC2DiffusionClients {
    storage: S3Storage,
    notification: Discord,
    diffusion: Box<dyn DiffusionClient>,
    engines: EC2MultiRegion,
    agent: Box<dyn ComputeAgent>,
}

impl EC2DiffusionClients {
    pub fn rc(env: &DiffusionEnv, config: SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(env, config))
    }

    fn agent(env: &DiffusionEnv) -> Box<dyn ComputeAgent> {
        if env.localhost {
            Box::new(LocalhostAgent::new())
        } else {
            Box::new(EC2Agent::new())
        }
    }

    fn new(env: &DiffusionEnv, config: SdkConfig) -> Self {
        let s3 = aws_sdk_s3::Client::new(&config);
        let own_region = config
            .region()
            .expect("AWS region could not be resolved (env var, profile, or EC2 IMDS)")
            .as_ref()
            .try_into()
            .expect("resolved AWS region is not a ComputeRegion this app knows about");
        let regions: Vec<ComputeRegion> = vec![];
        let ec2sdk = aws_sdk_ec2::Client::new(&config);
        let engines = EC2MultiRegion::new(regions, ec2sdk.clone());
        let general_clients = Self {
            storage: S3Storage::new(
                s3,
                own_region,
                env.output_bucket.clone(),
                env.output_prefix.clone(),
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

impl HasStorage for EC2DiffusionClients {
    type Storage = S3Storage;
    // type Engines = EC2MultiRegion;
    fn storage(&self) -> &Self::Storage {
        &self.storage
    }
}

impl HasComputeAgent for EC2DiffusionClients {
    fn agent(&self) -> &dyn ComputeAgent {
        self.agent.as_ref()
    }
}
