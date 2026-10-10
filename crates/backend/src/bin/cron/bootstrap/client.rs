use std::sync::Arc;

use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use aws_config::SdkConfig;

use backend::{
    application::{
        ports::clients::container::{HasNotification, HasOutputStorage, HasRaws, HasTranslator},
        usecases::translations::traits::ClientCtx,
    },
    infras::{
        http::internal_builder, notifications::discord::Discord, raws::syosetu::Syosetu,
        storage::s3::S3Storage, translators::chatgpt::ChatGPT,
    },
};
use domain::ctx;
use reqwest::header::HeaderMap;

use crate::env::CronEnv;

struct StaticClients {
    output_storage: S3Storage,
    translator: ChatGPT,
}
pub struct CronClients<'a, C: ClientCtx> {
    ctx: &'a C,
    env: &'a CronEnv,
    config: &'a SdkConfig,
    static_clients: Arc<StaticClients>,
}

impl<'a> CronClients<'a, ()> {
    pub fn arc(env: &'a CronEnv, config: &'a SdkConfig) -> Arc<Self> {
        Arc::new(Self::new(env, config))
    }

    pub fn new(env: &'a CronEnv, config: &'a SdkConfig) -> Self {
        let openai = OpenAIClient::<OpenAIConfig>::new();
        let static_clients: StaticClients = StaticClients {
            translator: ChatGPT::new(openai, &env.openai_model),
            output_storage: S3Storage::new(
                config.clone(),
                env.tl_region
                    .as_str()
                    .try_into()
                    .expect("env.tl_region must be a valid REGION"),
                env.tl_bucket.clone(),
                env.tl_rootdir.clone(),
                "/tmp/".into(),
                env.max_data_transfer,
                Some(aws_sdk_s3::types::ObjectCannedAcl::PublicRead),
            ),
        };
        Self {
            ctx: &(),
            env,
            config,
            static_clients: Arc::new(static_clients),
        }
    }

    pub fn start<'c: 'a>(&self, context: &'c ctx::Context) -> CronClients<'a, ctx::Context> {
        CronClients::<'a, ctx::Context> {
            ctx: context,
            env: self.env,
            config: self.config,
            static_clients: self.static_clients.clone(),
        }
    }
}

impl<'a, C: ClientCtx> HasTranslator for CronClients<'a, C> {
    type Translator = ChatGPT;
    fn translator(&self) -> &Self::Translator {
        &self.static_clients.translator
    }
}

impl<C: ClientCtx> HasOutputStorage for CronClients<'_, C> {
    type OutputStorage = S3Storage;
    fn output_storage(&self) -> &Self::OutputStorage {
        &self.static_clients.output_storage
    }
}

impl<'a> HasNotification for CronClients<'a, ctx::Context> {
    type Notification = Discord;
    fn notification(&self) -> Self::Notification {
        let additional = HeaderMap::new();
        let builder = internal_builder(self.ctx, additional);
        let client = builder.build().expect("must not fail");
        let env = self.env;
        Discord::new(
            client,
            env.discord_username.clone(),
            env.discord_webhook_url.clone(),
        )
    }
}
impl<'a> HasRaws for CronClients<'a, ctx::Context> {
    type Raws = Syosetu;
    fn raws(&self) -> Self::Raws {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36"));
        let mut builder = internal_builder(self.ctx, headers);
        // if let Some(p) = proxy {
        let env = self.env;
        let proxy_url = format!("http://{}:{}", env.proxy_host, env.proxy_port);
        let proxy = reqwest::Proxy::all(&proxy_url)
            .expect("invalid proxy url")
            .basic_auth(&env.proxy_username, &env.proxy_password);
        tracing::info!(
            "proxy_url: {}, username: {}",
            proxy_url,
            &env.proxy_username
        );
        builder = builder.proxy(proxy);
        let client = builder.build().expect("should not fail");
        Syosetu::new(client, env.syosetu_host.clone())
    }
}
