use std::{cell::Cell, rc::Rc};

use rust_api::{
    pkg::types::{peek::Peek, time::Timestamp},
    trigger::{
        commandq::CommandQ, idle_terminator::IdleTerminator, output_listener::NewOutputListener,
    },
};

mod bootstrap;
mod env;

use bootstrap::{client::EC2DiffusionClients, repo::EC2DiffusionRepo};
use env::DiffusionEnv;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    rust_api::init_env();
    rust_api::init_tracing();
    let env = DiffusionEnv::from_env();
    // make sure no missing env in the build
    tracing::debug!("sanity run: {}", env.sanity_run());
    if env.sanity_run() {
        return Ok(());
    }
    let queue_interval = env.queue_interval.clone();
    let track_interval = env.watch_interval.clone();
    let idle_tolerance = env.idle_tolerance.clone();
    let config = aws_config::from_env().load().await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);
    let watch_dir = env.watch_dir.clone();
    let repos = EC2DiffusionRepo::rc(&dynamo, env.stage());
    let clients = EC2DiffusionClients::rc(&env, config);
    let start_at = Timestamp::now();
    let rc_start_at = Rc::new(Cell::new(start_at));
    let rc_start_peek = Peek::new(rc_start_at.clone());
    let queue_handler = CommandQ::new(clients.clone(), repos.clone(), queue_interval);
    let output_listener = NewOutputListener::new(
        clients.clone(),
        repos.clone(),
        watch_dir,
        rc_start_at.clone(),
    );
    let idle_terminator = IdleTerminator::new(
        clients.clone(),
        repos.clone(),
        rc_start_peek,
        idle_tolerance,
        track_interval,
    );
    tokio::try_join!(
        queue_handler.run(),
        output_listener.run(),
        idle_terminator.run()
    )?;
    Ok(())
}
