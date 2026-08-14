use std::rc::Rc;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_api::{
    application::usecases::{
        bases::Usecase,
        translations::{init, run},
    },
    bootstrap::lambda::{client::LambdaClients, repo::LambdaRepos},
    config::lambda_env::LambdaEnv,
};
use serde::Deserialize;

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
    repo: Rc<LambdaRepos>,
    client: Rc<LambdaClients>,
    event: LambdaEvent<CronEvent>,
) -> Result<(), Error> {
    let path_params = &event.payload.path_parameters;
    let proxy = &path_params.proxy;
    match proxy.as_str() {
        "cron/translate" => {
            let params: run::Params = serde_json::from_value(path_params.data.clone())?;
            let action = run::Run::new(repo.clone(), client.clone(), params);
            action.exec().await?;
        }
        "cron/init" => {
            let params: init::Params = serde_json::from_value(path_params.data.clone())?;
            let action = init::Init::new(repo.clone(), client.clone(), params);
            action.exec().await?;
        }
        _ => tracing::warn!(proxy = proxy.as_str(), "unknown cron route"),
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    rust_api::init_env();
    rust_api::init_tracing();

    let env = LambdaEnv::from_env();
    let config = aws_config::from_env().load().await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);

    let repo = LambdaRepos::rc(&dynamo, env.stage());
    let client = LambdaClients::rc(env, config);

    run(service_fn(move |event| {
        let r = repo.clone();
        let c = client.clone();
        async move { handler(r, c, event).await }
    }))
    .await
}
