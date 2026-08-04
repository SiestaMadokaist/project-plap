pub mod resources;

use std::rc::Rc;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_api::{
    application::usecases::translations::{init, run},
    bootstrap::{client::GeneralClients, repo::GeneralRepositories},
    config::lambda_env::LambdaEnv,
};
use serde::Deserialize;

use crate::resources::translations::controller::TranslationController;

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
    repo: Rc<GeneralRepositories>,
    client: Rc<GeneralClients>,
    event: LambdaEvent<CronEvent>,
) -> Result<(), Error> {
    let path_params = &event.payload.path_parameters;
    let proxy = &path_params.proxy;
    let controller = TranslationController::new(repo, client);
    match proxy.as_str() {
        "cron/translate" => {
            let param: run::Params = serde_json::from_value(path_params.data.clone())?;
            controller.run(param).await?;
        }
        "cron/init" => {
            let param: init::Params = serde_json::from_value(path_params.data.clone())?;
            controller.init(param).await?;
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
    let config = aws_config::from_env()
        .region(aws_sdk_dynamodb::config::Region::new(env.region()))
        .load()
        .await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);

    let repo = GeneralRepositories::rc(&dynamo, &env);
    let client = GeneralClients::rc(env, config);

    run(service_fn(move |event| {
        let r = repo.clone();
        let c = client.clone();
        async move { handler(r, c, event).await }
    }))
    .await
}
