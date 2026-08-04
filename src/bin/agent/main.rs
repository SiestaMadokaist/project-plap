use std::{cell::Cell, rc::Rc};

use rust_api::{
    config::env::Env,
    pkg::types::time::{Second, Timestamp},
    trigger::{filewatch::Filewatch, terminator::Terminator},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    rust_api::init_env();
    rust_api::init_tracing();
    let env = Env::from_env();
    let dir = String::from("todo");
    let start = Timestamp::now();
    let rcc = Rc::new(Cell::new(start));
    let interval = Second(5);
    let tolerance = Second(60);
    let filewatch = Filewatch::new(dir, rcc.clone());
    let terminator = Terminator::new(rcc.clone(), tolerance, interval);

    filewatch.run();
    terminator.run();

    Ok(())
}
