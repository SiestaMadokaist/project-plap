use std::rc::Rc;

use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use aws_config::SdkConfig;

use crate::{
    application::ports::clients::{
        container::{AllClients, ClientContainer},
        diffusions::DiffusionClient,
    },
    config::env::Env,
    infras::{
        compute::ec2::EC2Compute,
        diffusions::a1111::A1111,
        notifications::discord::Discord,
        raws::syosetu::{ProxyConfig, Syosetu},
        storage::s3::S3Storage,
        translators::chatgpt::ChatGPT,
    },
};

pub struct GeneralClients {
    translator: ChatGPT,
    raws: Syosetu,
    storage: S3Storage,
    notification: Discord,
    diffusion: Box<dyn DiffusionClient>,
    compute: EC2Compute,
}

impl GeneralClients {
    pub fn rc(env: Env, config: SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(env, config))
    }

    fn new(env: Env, config: SdkConfig) -> Self {
        let s3 = aws_sdk_s3::Client::new(&config);
        let openai = OpenAIClient::<OpenAIConfig>::new();
        let proxy = match (
            env.proxy_host,
            env.proxy_port,
            env.proxy_username,
            env.proxy_password,
        ) {
            (Some(host), Some(port), Some(username), Some(password)) => Some(ProxyConfig {
                host,
                port: port.parse().expect("PROXY_PORT must be a number"),
                username,
                password,
            }),
            _ => None,
        };

        let s = Self {
            translator: ChatGPT::new(openai, &env.openai_model),
            raws: Syosetu::new(env.syosetu_host, proxy),
            storage: S3Storage::new(s3, env.tl_bucket, env.tl_prefix),
            notification: Discord::new(env.discord_webhook_url),
            // TODO: pick A1111 vs ComfyUI at runtime (e.g. from Env), not wired yet
            diffusion: Box::new(A1111::new(String::new())),
            // TODO: not wired to Env yet, stub only
            compute: EC2Compute::new(aws_sdk_ec2::Client::new(&config), String::new()),
        };
        s
    }
}

impl ClientContainer for GeneralClients {
    type Translator = ChatGPT;
    type Raws = Syosetu;
    type Storage = S3Storage;
    type Notification = Discord;
    type Compute = EC2Compute;

    fn translator(&self) -> &Self::Translator {
        &self.translator
    }
    fn raws(&self) -> &Self::Raws {
        &self.raws
    }
    fn storage(&self) -> &Self::Storage {
        &self.storage
    }
    fn notification(&self) -> &Self::Notification {
        &self.notification
    }
    fn diffusion(&self) -> &dyn DiffusionClient {
        self.diffusion.as_ref()
    }

    fn compute(&self) -> &Self::Compute {
        &self.compute
    }
}

impl AllClients for GeneralClients {}
