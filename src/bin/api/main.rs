use std::{collections::HashMap, rc::Rc};

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use matchit::Router;
use rust_api::{
    bootstrap::cron::{client::CronClients, repo::CronRepos},
    config::cron_env::CronEnv,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ApiEvent {
    path: String,
    #[serde(rename = "httpMethod")]
    #[allow(dead_code)]
    http_method: String,
    #[allow(dead_code)]
    body: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse {
    #[serde(rename = "statusCode")]
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

fn json_response(status_code: u16, body: impl Into<String>) -> ApiResponse {
    let mut headers = HashMap::new();
    headers.insert("content-type".into(), "application/json".into());
    ApiResponse {
        status_code,
        headers,
        body: body.into(),
    }
}

enum RouteId {
    Hello,
}

fn routes() -> Router<RouteId> {
    let mut router = Router::new();
    router.insert("/", RouteId::Hello).expect("valid route");
    router
}

async fn hello(_repo: &CronRepos, _client: &CronClients) -> ApiResponse {
    json_response(200, r#"{"message":"hello world"}"#)
}

async fn handler(
    repo: Rc<CronRepos>,
    client: Rc<CronClients>,
    router: Rc<Router<RouteId>>,
    event: LambdaEvent<ApiEvent>,
) -> Result<ApiResponse, Error> {
    let path = &event.payload.path;

    let resp = match router.at(path) {
        Ok(matched) => match matched.value {
            RouteId::Hello => hello(&repo, &client).await,
        },
        Err(_) => json_response(404, r#"{"error":"not found"}"#),
    };

    Ok(resp)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    rust_api::init_env();
    rust_api::init_tracing();

    let env = CronEnv::from_env();
    let config = aws_config::from_env().load().await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);

    let repo = CronRepos::rc(&dynamo, env.stage());
    let client = CronClients::rc(env, config);
    let router = Rc::new(routes());

    run(service_fn(move |event| {
        let r = repo.clone();
        let c = client.clone();
        let rt = router.clone();
        async move { handler(r, c, rt, event).await }
    }))
    .await
}
