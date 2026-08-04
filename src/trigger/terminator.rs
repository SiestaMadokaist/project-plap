use crate::pkg::types::time::{Second, Timestamp};
use std::{cell::Cell, rc::Rc};

/**
 * auto terminator that check if last_ok timestamp has passed its threshold
 * it'll automatically request to shutdown the compute instance
 */
pub struct Terminator {
    last_ok: Rc<Cell<Timestamp>>,
    tolerance: Second,
    interval: Second,
}

impl Terminator {
    pub fn new(start_at: Rc<Cell<Timestamp>>, tolerance: Second, interval: Second) -> Self {
        Self {
            last_ok: start_at,
            tolerance,
            interval,
        }
    }
    async fn on_interval(&self) -> anyhow::Result<()> {
        let tolerance = &self.tolerance;
        let now = Timestamp::now();
        let last_ok = &self.last_ok.get();
        let delta = now.sub(last_ok);
        if delta.gt(tolerance) {
            self.terminate().await?;
        }
        Ok(())
    }

    async fn terminate(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}
