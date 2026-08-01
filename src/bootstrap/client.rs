use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use aws_config::SdkConfig;

use crate::{
    application::ports::clients::ctrait::{AllClients, ClientContainer},
    config::env::Env,
    infras::{
        notifications::discord::Discord,
        raws::syosetu::{ProxyConfig, Syosetu},
        storage::s3::S3Storage,
        translators::chatgpt::ChatGPT,
    },
};

pub struct CronClientContainer {
    translator: ChatGPT,
    raws: Syosetu,
    storage: S3Storage,
    notification: Discord,
}

impl CronClientContainer {
    pub fn new(env: Env, config: SdkConfig) -> Self {
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

        Self {
            translator: ChatGPT::new(openai, &env.openai_model),
            raws: Syosetu::new(env.syosetu_host, proxy),
            storage: S3Storage::new(s3, env.tl_bucket, env.tl_prefix),
            notification: Discord::new(env.discord_webhook_url),
        }
    }
}

impl ClientContainer for CronClientContainer {
    type Translator = ChatGPT;
    type Raws = Syosetu;
    type Storage = S3Storage;
    type Notification = Discord;

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
}

impl AllClients for CronClientContainer {}
