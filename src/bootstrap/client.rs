use std::env;

use async_openai::{config::OpenAIConfig, Client as OpenAIClient};

use crate::{
    application::ports::clients::cc::{AllClient, ClientContainer},
    infras::{
        notifications::discord::Discord,
        raws::syosetu::Syosetu,
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
    pub fn new(s3: aws_sdk_s3::Client) -> Self {
        let openai = OpenAIClient::<OpenAIConfig>::new();
        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".into());
        let syosetu_host =
            env::var("SYOSETU_HOST").unwrap_or_else(|_| "https://ncode.syosetu.com".into());
        let bucket = env::var("TL_BUCKET").expect("TL_BUCKET must be set");
        let prefix = env::var("TL_PREFIX").unwrap_or_else(|_| "".into());
        let webhook_url = env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL must be set");

        Self {
            translator: ChatGPT::new(openai, &model),
            raws: Syosetu::new(syosetu_host),
            storage: S3Storage::new(s3, bucket, prefix),
            notification: Discord::new(webhook_url),
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

impl AllClient for CronClientContainer {}
