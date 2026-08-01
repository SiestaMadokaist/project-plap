use serde::{Deserialize, Serialize};

use crate::pkg::types::unit;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct InferenceProgress {
    n_iter: unit::Index1,
    progress: unit::Index1,
}

impl InferenceProgress {
    pub fn next(&mut self) -> () {
        self.progress.next()
    }
}

#[derive(Serialize, Deserialize)]
pub struct InferenceConfig {
    positive: String,
    negative: String,
    width: unit::Px,
    height: unit::Px,
    steps: unit::Index1,
    seed: u32,
}

#[derive(Serialize, Deserialize)]
pub struct InferenceArgs {
    pub progress: InferenceProgress,
    pub config: InferenceConfig,
}
