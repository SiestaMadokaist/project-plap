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
    novel_id: Option<NovelId>,
}

async fn handler(event: LambdaEvent<CronEvent>) -> Result<(), Error> {
    let param = &event.payload.path_parameters;
    let proxy = &param.proxy;
    let novel_id = &param.novel_id;
    tracing::info!(proxy = proxy.as_str(), "cron event");

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

    // controller
    //     .init(init::Params {
    //         novel_id: NovelId("n4449cj".into()),
    //         starting_chapter: Some(ChapterId(131.into())),
    //         title: String::from("deathflag"),
    //     })
    //     .await?;

    // controller
    //     .init(init::Params {
    //         novel_id: NovelId("https://ncode.syosetu.com/n2267be".into()),
    //         starting_chapter: Some(ChapterId(131.into())),
    //         title: String::from("rezero"),
    //     })
    //     .await?;

    // Ok(())
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
        "cron/init" => match novel_id {
            None => Err("novel_id is required for cron/translate".into()),
            Some(id) => {
                let _result = controller
                    .init(init::Params {
                        novel_id: id.clone(),
                        starting_chapter: Some(ChapterId(100)),
                        title: String::from("idk"),
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
    rust_api::init_env();
    rust_api::init_tracing();
    run(service_fn(handler)).await
}
