use std::rc::Rc;

use backend::application::{
    ports::usecase::UsecaseAPI,
    usecases::translations::{init, run},
};
use domain::ctx;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Deserialize;

mod bootstrap;
mod env;

use bootstrap::{client::CronClients, repo::CronRepos};
use env::CronEnv;

#[derive(Deserialize)]
struct CronEvent {
    #[serde(rename = "pathParameters")]
    path_parameters: PathParameters,
}

#[derive(Deserialize)]
struct PathParameters {
    proxy: String,
    data: serde_json::Value,
}

async fn handler(
    repo: Rc<CronRepos>,
    client: &CronClients<'_, ()>,
    event: LambdaEvent<CronEvent>,
) -> Result<(), Error> {
    let path_params = &event.payload.path_parameters;
    let proxy = &path_params.proxy;
    let context = &ctx::Context::guest(None);
    let ctx_client = client.start(context);
    match proxy.as_str() {
        "cron/translate" => {
            let params: dto::resources::translations::RunPayload =
                serde_json::from_value(path_params.data.clone())?;

            let action = run::Run::new(repo.as_ref(), &ctx_client, params);
            action.exec().await?;
        }
        "cron/init" => {
            let params: dto::resources::translations::InitPayload =
                serde_json::from_value(path_params.data.clone())?;
            let action = init::Init::new(repo.as_ref(), &ctx_client, params);
            action.exec().await?;
        }
        _ => tracing::warn!(proxy = proxy.as_str(), "unknown cron route"),
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    backend::init_env();
    let env = CronEnv::from_env();
    let _sentry = backend::init_sentry(env.sentry_dsn.as_deref(), &env.sentry_env);
    backend::init_tracing();

    let config = aws_config::from_env().load().await;

    let repo = CronRepos::rc(&config, env.stage());
    let client = CronClients::rc(&env, &config);

    run(service_fn(move |event| {
        let r = repo.clone();
        let c = client.clone();
        async move { handler(r, &c, event).await }
    }))
    .await
}
