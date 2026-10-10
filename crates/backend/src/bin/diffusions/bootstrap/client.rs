use std::sync::Arc;

use aws_config::SdkConfig;
use aws_sdk_s3::types::ObjectCannedAcl::PublicRead;
use domain::ctx;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::env::DiffusionEnv;
use backend::{
    application::{
        ports::clients::{
            compute_agent::ComputeAgent,
            container::{
                HasCivitai, HasComputeAgent, HasDiffusion, HasEngines, HasModelStorage,
                HasNotification, HasOutputStorage,
            },
            diffusions::DiffusionClient,
            inference_model_provider::InferenceModelProvider,
        },
        usecases::translations::traits::ClientCtx,
    },
    infras::{
        civitai::client::CivitaiAPI,
        compute::ec2::EC2MultiRegion,
        compute_agent::{ec2::EC2Agent, localhost::LocalhostAgent},
        diffusions::comfyui::ComfyUI,
        http::internal_builder,
        notifications::discord::Discord,
        storage::s3::S3Storage,
    },
};

/// Clients that don't depend on the request's `ctx::Context`, built once and shared.
struct StaticClients {
    output_storage: S3Storage,
    model_storage: S3Storage,
    engines: EC2MultiRegion,
}

impl StaticClients {
    fn new(env: &DiffusionEnv, config: &SdkConfig) -> Self {
        Self {
            output_storage: S3Storage::new(
                config.clone(),
                env.output_region
                    .as_str()
                    .try_into()
                    .expect("env.output_region must be a valid region"),
                env.output_bucket.clone(),
                env.output_rootdir.clone(),
                env.workdir.clone(),
                env.max_data_transfer,
                Some(PublicRead),
            ),
            model_storage: S3Storage::new(
                config.clone(),
                env.model_region
                    .as_str()
                    .try_into()
                    .expect("env.model_region must be a valid region"),
                env.model_bucket.clone(),
                env.model_rootdir.clone(),
                env.workdir.clone(),
                env.max_data_transfer,
                None,
            ),
            engines: EC2MultiRegion::new(config.clone()),
        }
    }
}

pub struct EC2DiffusionClients<'a, C: ClientCtx> {
    ctx: &'a C,
    env: &'a DiffusionEnv,
    config: &'a SdkConfig,
    static_clients: Arc<StaticClients>,
}

impl<'a> EC2DiffusionClients<'a, ()> {
    pub fn new(env: &'a DiffusionEnv, config: &'a SdkConfig) -> Self {
        Self {
            ctx: &(),
            env,
            config,
            static_clients: Arc::new(StaticClients::new(env, config)),
        }
    }

    pub fn start(&self, context: &'a ctx::Context) -> EC2DiffusionClients<'a, ctx::Context> {
        EC2DiffusionClients::<'a, ctx::Context> {
            ctx: context,
            env: self.env,
            config: self.config,
            static_clients: self.static_clients.clone(),
        }
    }
}

fn agent(env: &DiffusionEnv, config: &SdkConfig) -> Box<dyn ComputeAgent> {
    if env.localhost {
        Box::new(LocalhostAgent::new())
    } else {
        Box::new(EC2Agent::new(config))
    }
}

impl<C: ClientCtx> HasEngines for EC2DiffusionClients<'_, C> {
    type Engines = EC2MultiRegion;
    fn engines(&self) -> &Self::Engines {
        &self.static_clients.engines
    }
}

impl<C: ClientCtx> HasModelStorage for EC2DiffusionClients<'_, C> {
    type ModelStorage = S3Storage;
    fn model_storage(&self) -> &Self::ModelStorage {
        &self.static_clients.model_storage
    }
}

impl<C: ClientCtx> HasOutputStorage for EC2DiffusionClients<'_, C> {
    type OutputStorage = S3Storage;
    fn output_storage(&self) -> &Self::OutputStorage {
        &self.static_clients.output_storage
    }
}

impl<C: ClientCtx> HasComputeAgent for EC2DiffusionClients<'_, C> {
    fn agent(&self) -> Box<dyn ComputeAgent> {
        agent(self.env, self.config)
    }
}

impl HasDiffusion for EC2DiffusionClients<'_, ctx::Context> {
    fn diffusion(&self) -> Box<dyn DiffusionClient> {
        let additional = HeaderMap::new();
        let builder = internal_builder(self.ctx, additional);
        let client = builder.build().expect("client build must not fail");
        let comfyui = ComfyUI::new(client, self.env.inference_host.clone());
        Box::new(comfyui)
    }
}

impl HasNotification for EC2DiffusionClients<'_, ctx::Context> {
    type Notification = Discord;
    fn notification(&self) -> Self::Notification {
        let additional = HeaderMap::new();
        let builder = internal_builder(self.ctx, additional);
        let client = builder.build().expect("client build must not fail");
        Discord::new(
            client,
            self.env.discord_username.clone(),
            self.env.discord_webhook_url.clone(),
        )
    }
}

impl HasCivitai for EC2DiffusionClients<'_, ctx::Context> {
    fn civitai(&self) -> Box<dyn InferenceModelProvider> {
        let mut additional = HeaderMap::new();
        let bearer = format!("Bearer {}", self.env.civitai_apikey);
        if let Ok(value) = HeaderValue::from_str(&bearer) {
            additional.insert(reqwest::header::AUTHORIZATION, value);
        }
        let builder = internal_builder(self.ctx, additional);
        let client = builder.build().expect("client build must not fail");
        Box::new(CivitaiAPI::new(
            client,
            self.env.civitai_host(),
            self.env.workdir.clone(),
        ))
    }
}
