use std::rc::Rc;

use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use aws_config::SdkConfig;

use crate::{
    application::ports::clients::container::{
        HasEngines, HasModelStorage, HasNotification, HasOutputStorage, HasRaws, HasTranslator,
    },
    config::lambda_env::LambdaEnv,
    domain::commands::compute::ComputeRegion,
    infras::{
        compute::ec2::EC2MultiRegion,
        notifications::discord::Discord,
        raws::syosetu::{ProxyConfig, Syosetu},
        storage::s3::S3Storage,
        translators::chatgpt::ChatGPT,
    },
};

pub struct LambdaClients {
    translator: ChatGPT,
    raws: Syosetu,
    model_storage: S3Storage,
    output_storage: S3Storage,
    notification: Discord,
    engines: EC2MultiRegion,
}

impl LambdaClients {
    pub fn rc(env: LambdaEnv, config: SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(env, config))
    }

    fn new(env: LambdaEnv, config: SdkConfig) -> Self {
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

        let regions: Vec<ComputeRegion> = vec![];
        let ec2sdk = aws_sdk_ec2::Client::new(&config);
        let engines = EC2MultiRegion::new(regions, ec2sdk.clone());

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
            model_storage: S3Storage::new(
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
            // TODO: not wired to Env yet, stub only
            engines,
        };
        general_clients
    }
}

impl HasTranslator for LambdaClients {
    type Translator = ChatGPT;
    fn translator(&self) -> &Self::Translator {
        &self.translator
    }
}

impl HasRaws for LambdaClients {
    type Raws = Syosetu;
    fn raws(&self) -> &Self::Raws {
        &self.raws
    }
}

impl HasModelStorage for LambdaClients {
    type ModelStorage = S3Storage;
    fn model_storage(&self) -> &Self::ModelStorage {
        &self.model_storage
    }
}

impl HasOutputStorage for LambdaClients {
    type OutputStorage = S3Storage;
    fn output_storage(&self) -> &Self::OutputStorage {
        &self.output_storage
    }
}

impl HasNotification for LambdaClients {
    type Notification = Discord;
    fn notification(&self) -> &Self::Notification {
        &self.notification
    }
}

impl HasEngines for LambdaClients {
    type Engines = EC2MultiRegion;
    fn engines(&self) -> &Self::Engines {
        &self.engines
    }
}
