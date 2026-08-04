use std::{cell::Cell, rc::Rc};

use rust_api::{
    bootstrap::{client::GeneralClients, repo::GeneralRepositories},
    config::lambda_env::LambdaEnv,
    pkg::types::{
        peek::Peek,
        time::{Second, Timestamp},
    },
    trigger::{activity_tracker::ActivityTracker, idle_terminator::IdleTerminator},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    rust_api::init_env();
    rust_api::init_tracing();
    let env = LambdaEnv::from_env();
    let config = aws_config::from_env()
        .region(aws_sdk_dynamodb::config::Region::new(env.region()))
        .load()
        .await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);

    let repos = GeneralRepositories::rc(&dynamo, &env);
    let clients = GeneralClients::rc(env, config);
    let dir = String::from("todo");
    let start_at = Timestamp::now();
    let rc_start_at = Rc::new(Cell::new(start_at));
    let rc_start_peek = Peek::new(rc_start_at.clone());
    let interval = Second(5);
    let tolerance = Second(60);
    let activity_tracker =
        ActivityTracker::new(clients.clone(), repos.clone(), dir, rc_start_at.clone());
    let idle_terminator = IdleTerminator::new(clients.clone(), rc_start_peek, tolerance, interval);
    tokio::try_join!(activity_tracker.run(), idle_terminator.run())?;

    Ok(())
}
