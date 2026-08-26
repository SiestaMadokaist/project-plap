use std::rc::Rc;

use backend::application::{
    ports::usecase::UsecaseAPI,
    usecases::translations::{init, run},
};
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
    client: Rc<CronClients>,
    event: LambdaEvent<CronEvent>,
) -> Result<(), Error> {
    let path_params = &event.payload.path_parameters;
    let proxy = &path_params.proxy;
    match proxy.as_str() {
        "cron/translate" => {
            let params: dto::resources::translations::RunPayload =
                serde_json::from_value(path_params.data.clone())?;
            let action = run::Run::new(repo.clone(), client.clone(), params);
            action.exec().await?;
        }
        "cron/init" => {
            let params: dto::resources::translations::InitPayload =
                serde_json::from_value(path_params.data.clone())?;
            let action = init::Init::new(repo.clone(), client.clone(), params);
            action.exec().await?;
        }
        _ => tracing::warn!(proxy = proxy.as_str(), "unknown cron route"),
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    backend::init_env();
    backend::init_tracing();

    let env = CronEnv::from_env();
    let config = aws_config::from_env().load().await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);

    let repo = CronRepos::rc(&dynamo, env.stage());
    let client = CronClients::rc(env, config);

    run(service_fn(move |event| {
        let r = repo.clone();
        let c = client.clone();
        async move { handler(r, c, event).await }
    }))
    .await
}
