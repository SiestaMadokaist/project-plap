pub mod resources;

use std::rc::Rc;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_api::{
    application::usecases::translations::run,
    bootstrap::{client::CronClientContainer, repo::DynamoRepositoryContainer},
    domain::translation::NovelId,
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
    novel_id: Option<NovelId>,
}

async fn handler(event: LambdaEvent<CronEvent>) -> Result<(), Error> {
    let param = &event.payload.path_parameters;
    let proxy = &param.proxy;
    let novel_id = &param.novel_id;
    tracing::info!(proxy = proxy.as_str(), "cron event");

    let config = aws_config::load_from_env().await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);
    let s3 = aws_sdk_s3::Client::new(&config);
    let repo = Rc::new(DynamoRepositoryContainer::new(
        DDBTranslationRepository::new(dynamo.clone()),
        DDBUserRepository::new(dynamo),
    ));
    let client = Rc::new(CronClientContainer::new(s3));
    let controller = TranslationController::new(repo, client);

    match proxy.as_str() {
        "cron/translate" => match novel_id {
            None => Err("novel_id is required for cron/translate".into()),
            Some(id) => {
                controller
                    .run(run::Params {
                        novel_id: id.clone(),
                    })
                    .await?;
                Ok(())
            }
        },
        _ => {
            tracing::warn!(proxy = proxy.as_str(), "unknown cron route");
            Ok(())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    rust_api::init_tracing();
    run(service_fn(handler)).await
}
