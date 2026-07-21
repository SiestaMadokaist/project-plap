pub mod resources;

use std::rc::Rc;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_api::{
    application::usecases::translations::{init, run},
    bootstrap::{client::CronClientContainer, repo::DynamoRepositoryContainer},
    domain::translation::{ChapterId, NovelId},
    infras::repos::dynamo::{translation::DDBTranslationRepository, user::DDBUserRepository},
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

async fn handler(event: LambdaEvent<CronEvent>) -> Result<(), Error> {
    let path_params = &event.payload.path_parameters;
    let proxy = path_params.proxy;
    let region = std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "ap-southeast-1".into());
    let config = aws_config::from_env()
        .region(aws_sdk_dynamodb::config::Region::new(region))
        .load()
        .await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);
    let s3 = aws_sdk_s3::Client::new(&config);
    let repo = Rc::new(DynamoRepositoryContainer::new(
        DDBTranslationRepository::new(dynamo.clone()),
        DDBUserRepository::new(dynamo),
    ));
    let client = Rc::new(CronClientContainer::new(s3));
    let controller = TranslationController::new(repo, client);
    match proxy.as_str() {
        "cron/translate" => {
            let param: run::Params = serde_json::from_value(path_params.data)?;
            let result = controller.run(param).await?;
            Ok(())
        }
        "cron/init" => {
            let param: init::Params = serde_json::from_value(path_params.data)?;
            let result = controller.init(param);
            Ok(())
        }
        _ => {
            tracing::warn!(proxy = proxy.as_str(), "unknown cron route");
            Ok(())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    rust_api::init_env();
    rust_api::init_tracing();
    run(service_fn(handler)).await
}
