use std::rc::Rc;

use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use aws_config::SdkConfig;

use rust_api::{
    application::ports::clients::container::{
        HasNotification, HasOutputStorage, HasRaws, HasTranslator,
    },
    infras::{
        notifications::discord::Discord,
        raws::syosetu::{ProxyConfig, Syosetu},
        storage::s3::S3Storage,
        translators::chatgpt::ChatGPT,
    },
};

use crate::env::CronEnv;

pub struct CronClients {
    translator: ChatGPT,
    raws: Syosetu,
    output_storage: S3Storage,
    notification: Discord,
}

impl CronClients {
    pub fn rc(env: CronEnv, config: SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(env, config))
    }

    fn new(env: CronEnv, config: SdkConfig) -> Self {
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
        let general_clients = Self {
            translator: ChatGPT::new(openai, &env.openai_model),
            raws: Syosetu::new(env.syosetu_host, proxy),
            output_storage: S3Storage::new(
                config.clone(),
                env.tl_region
                    .as_str()
                    .try_into()
                    .expect("env.tl_region must be a valid REGION"),
                env.tl_bucket.clone(),
                env.tl_prefix.clone(),
                env.max_data_transfer,
            ),
            notification: Discord::new(env.discord_webhook_url),
        };
        general_clients
    }
}

impl HasTranslator for CronClients {
    type Translator = ChatGPT;
    fn translator(&self) -> &Self::Translator {
        &self.translator
    }
}

impl HasRaws for CronClients {
    type Raws = Syosetu;
    fn raws(&self) -> &Self::Raws {
        &self.raws
    }
}

impl HasOutputStorage for CronClients {
    type OutputStorage = S3Storage;
    fn output_storage(&self) -> &Self::OutputStorage {
        &self.output_storage
    }
}

impl HasNotification for CronClients {
    type Notification = Discord;
    fn notification(&self) -> &Self::Notification {
        &self.notification
    }
}
