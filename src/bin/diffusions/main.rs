use std::{cell::Cell, rc::Rc};

use rust_api::{
    application::ports::clients::container::HasTranslator,
    bootstrap::ec2diffusion::{client::EC2DiffusionClients, repo::EC2DiffusionRepo},
    config::diffusion_env::DiffusionEnv,
    pkg::types::{
        peek::Peek,
        time::{Second, Timestamp},
    },
    trigger::{
        commandq::CommandQ, idle_terminator::IdleTerminator, newimage_listener::NewImageListener,
    },
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    rust_api::init_env();
    rust_api::init_tracing();
    let env = DiffusionEnv::from_env();
    let config = aws_config::from_env().load().await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);
    let watch_dir = env.watch_dir.clone();
    let repos = EC2DiffusionRepo::rc(&dynamo, env.stage());
    let clients = EC2DiffusionClients::rc(env, config);
    let c = clients.clone();
    let start_at = Timestamp::now();
    let rc_start_at = Rc::new(Cell::new(start_at));
    let rc_start_peek = Peek::new(rc_start_at.clone());
    let minute = 60;
    let queue_interval = Second(20);
    let queue_handler = CommandQ::new(clients.clone(), repos.clone(), queue_interval);
    let track_interval = Second(15 * minute);
    let idle_tolerance = Second(60 * minute);
    let activity_tracker = NewImageListener::new(
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
        activity_tracker.run(),
        idle_terminator.run()
    )?;
    Ok(())
}
