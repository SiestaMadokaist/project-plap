use std::{cell::Cell, rc::Rc};

use crate::pkg::types::time::Timestamp;

pub struct Filewatch {
    dir: String,
    last_change: Rc<Cell<Timestamp>>,
}

impl Filewatch {
    pub fn new(dir: String, last_change: Rc<Cell<Timestamp>>) -> Self {
        Self { dir, last_change }
    }
    // filter if the change is something we care about.
    fn changed(&self) -> bool {
        true
    }

    /**
     * @todo:
     * extract exif from image
     * store exif to bigquery
     * upload to s3
     */
    fn handle_change(&self) -> anyhow::Result<()> {
        todo!();
    }

    fn on_change(&mut self) -> anyhow::Result<()> {
        let now = Timestamp::now();
        self.last_change.set(now);
        self.handle_change()
    }

    pub fn run(&self) -> anyhow::Result<()> {
        todo!();
    }
}
