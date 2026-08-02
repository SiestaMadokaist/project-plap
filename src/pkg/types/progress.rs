use serde::{Deserialize, Serialize};

use crate::pkg::types::unit;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Progression {
    total: unit::Index1,
    progress: unit::Index1,
    started: bool,
}

impl Progression {
    pub fn increment(&mut self) -> () {
        self.progress.next()
    }

    pub fn is_done(&self) -> bool {
        return self.progress == self.total;
    }
}
