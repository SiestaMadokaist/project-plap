use std::rc::Rc;

use crate::env::ApiEnv;
use aws_config::SdkConfig;
use backend::{
    application::{
        ports::clients::container::{
            HasAuthValidator, HasEngines, HasModelStorage, HasNotification, HasOutputStorage,
        },
        usecases::translations::traits::ClientCtx,
    },
    infras::{
        authorizer::client::EthAuth, compute::ec2::EC2MultiRegion, http::internal_api,
        notifications::discord::Discord, storage::s3::S3Storage,
    },
};
use domain::ctx;

#[derive(Debug)]
struct InternalClients {
    engines: EC2MultiRegion,
    output_storage: S3Storage,
    model_storage: S3Storage,
    authorizer: EthAuth,
}

pub struct ApiClients<'a, C: ClientCtx> {
    ctx: &'a C,
    env: &'a ApiEnv,
    config: &'a SdkConfig,
    internal: Rc<InternalClients>,
}

const MAX_SIZE: i64 = 100 * 1024 * 1024;

impl InternalClients {
    pub fn rc(env: &ApiEnv, config: &SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(env, config))
    }

    pub fn new(env: &ApiEnv, config: &SdkConfig) -> Self {
        let engines = EC2MultiRegion::new(config.clone());
        Self {
            engines,
            output_storage: S3Storage::new(
                config.clone(),
                env.output_region
                    .as_str()
                    .try_into()
                    .expect("env.output_region must be a valid region"),
                env.output_bucket.clone(),
                env.output_rootdir.clone(),
                "/tmp/".into(),
                MAX_SIZE,
                None,
            ),
            model_storage: S3Storage::new(
                config.clone(),
                env.model_region
                    .as_str()
                    .try_into()
                    .expect("env.model_region must be a valid region"),
                env.model_bucket.clone(),
                env.model_rootdir.clone(),
                "/tmp/".into(),
                MAX_SIZE,
                None,
            ),
            authorizer: EthAuth::new(
                env.auth_secret.clone(),
                env.auth_privkey.clone(),
                env.auth_challenge_ttl,
                env.auth_session_ttl,
                env.auth_clock_skew,
                env.auth_min_iat,
            )
            .expect("AUTH_PRIVKEY must be a valid secp256k1 scalar"),
        }
    }
}

impl<'a, C: ClientCtx> ApiClients<'a, C> {
    pub fn rc(ctx: &'a C, env: &'a ApiEnv, config: &'a SdkConfig) -> Rc<Self> {
        Rc::new(Self::new(ctx, env, config))
    }

    pub fn new(ctx: &'a C, env: &'a ApiEnv, config: &'a SdkConfig) -> Self {
        let internal = InternalClients::rc(env, config);
        Self {
            ctx,
            env,
            config,
            internal,
        }
    }
}

impl<'a> ApiClients<'a, ()> {
    pub fn start(&self, ctx: &'a ctx::Context) -> ApiClients<'a, ctx::Context> {
        ApiClients {
            ctx,
            env: self.env,
            config: self.config,
            internal: self.internal.clone(),
        }
    }
}

impl<'a> HasNotification for ApiClients<'a, ctx::Context> {
    type Notification = Discord;
    fn notification(&self) -> Self::Notification {
        let client = internal_api(&self.ctx);
        Discord::new(
            client,
            self.env.discord_username.clone(),
            self.env.discord_webhook_url.clone(),
        )
    }
}

impl<A: ClientCtx> HasEngines for ApiClients<'_, A> {
    type Engines = EC2MultiRegion;
    fn engines(&self) -> &Self::Engines {
        &self.internal.engines
    }
}

impl<A: ClientCtx> HasOutputStorage for ApiClients<'_, A> {
    type OutputStorage = S3Storage;
    fn output_storage(&self) -> &Self::OutputStorage {
        &self.internal.output_storage
    }
}

impl<A: ClientCtx> HasModelStorage for ApiClients<'_, A> {
    type ModelStorage = S3Storage;
    fn model_storage(&self) -> &Self::ModelStorage {
        &self.internal.model_storage
    }
}

impl<A: ClientCtx> HasAuthValidator for ApiClients<'_, A> {
    type Auth = EthAuth;
    fn authorizer(&self) -> &Self::Auth {
        &self.internal.authorizer
    }
}
