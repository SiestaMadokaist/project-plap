use std::rc::Rc;

use crate::env::ApiEnv;
use aws_config::SdkConfig;
use backend::{
    application::ports::clients::container::{
        HasAuthValidator, HasEngines, HasModelStorage, HasNotification, HasOutputStorage,
    },
    infras::{
        authorizer::client::EthAuth, compute::ec2::EC2MultiRegion, notifications::discord::Discord,
        storage::s3::S3Storage,
    },
};
use domain::{ctx, errors::DomainError};

pub trait Ctx {}
impl Ctx for () {}
impl Ctx for ctx::Context {}

// #[derive(Debug)]
pub struct ApiClients<'a, C: Ctx> {
    ctx: C,
    env: &'a ApiEnv,
    config: &'a SdkConfig,
    // notification: Discord,
    // engines: EC2MultiRegion,
    // output_storage: S3Storage,
    // model_storage: S3Storage,
    // authorizer: EthAuth,
}

const MAX_SIZE: i64 = 100 * 1024 * 1024;

impl<'a, C: Ctx> ApiClients<'a, C> {
    pub fn rc(ctx: C, env: &'a ApiEnv, config: &'a SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(ctx, env, config))
    }

    pub fn new(ctx: C, env: &'a ApiEnv, config: &'a SdkConfig) -> Self {
        Self { ctx, env, config }
        // let engines = EC2MultiRegion::new(config.clone());
        // Self {
        //     notification: Discord::new(
        //         env.discord_username.clone(),
        //         env.discord_webhook_url.clone(),
        //     ),
        //     engines,
        //     output_storage: S3Storage::new(
        //         config.clone(),
        //         env.output_region
        //             .as_str()
        //             .try_into()
        //             .expect("env.output_region must be a valid region"),
        //         env.output_bucket.clone(),
        //         env.output_rootdir.clone(),
        //         "/tmp/".into(),
        //         MAX_SIZE,
        //         None,
        //     ),
        //     model_storage: S3Storage::new(
        //         config.clone(),
        //         env.model_region
        //             .as_str()
        //             .try_into()
        //             .expect("env.model_region must be a valid region"),
        //         env.model_bucket.clone(),
        //         env.model_rootdir.clone(),
        //         "/tmp/".into(),
        //         MAX_SIZE,
        //         None,
        //     ),
        //     authorizer: EthAuth::new(
        //         env.auth_secret.clone(),
        //         env.auth_privkey.clone(),
        //         env.auth_challenge_ttl,
        //         env.auth_session_ttl,
        //         env.auth_clock_skew,
        //         env.auth_min_iat,
        //     )
        //     .expect("AUTH_PRIVKEY must be a valid secp256k1 scalar"),
        // }
    }
}

impl<'a> ApiClients<'a, ()> {
    pub fn start(&self, ctx: ctx::Context) -> ApiClients<'a, ctx::Context> {
        ApiClients {
            ctx,
            env: self.env,
            config: self.config,
        }
    }
}

impl<'a> ApiClients<'a, ctx::Context> {}

impl<'a> HasNotification for ApiClients<'a, ctx::Context> {
    type Notification = Discord;
    fn notification(&self) -> Result<Self::Notification, DomainError> {
        // let client = self.request()?;
        let notification = Discord::new(
            client,
            self.env.discord_username.clone(),
            self.env.discord_webhook_url.clone(),
        );
        Ok(notification)
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

impl HasAuthValidator for ApiClients {
    type Auth = EthAuth;
    fn authorizer(&self) -> &Self::Auth {
        &self.authorizer
    }
}
