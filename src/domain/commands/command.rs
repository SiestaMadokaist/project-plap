use serde::{Deserialize, Serialize};

use crate::{
    domain::commands::{compute::ComputeArgs, inference::InferenceArgs, network::NetworkArgs},
    pkg::{
        macros::id_type,
        types::{
            time::{self, Timestamp},
            unit,
        },
    },
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStage {
    InProgress,
    // Running is skipped
    // we use InProgress { started_at: Some } to mark its Running
    Completed,
}

impl From<CommandStage> for String {
    fn from(value: CommandStage) -> Self {
        let s: &str = match value {
            CommandStage::InProgress => "in_progress",
            CommandStage::Completed => "completed",
        };
        String::from(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    S3,
    Civitai,
    Https,
}
id_type!(ActionID);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Progression {
    total: unit::Index1,
    progress: unit::Index1,
    started_at: Option<Timestamp>,
    finished_at: Option<Timestamp>,
}

impl Progression {
    pub fn increment(&mut self) -> () {
        self.progress.next();
        if self.is_done() {
            self.finished_at = Some(time::Timestamp::now());
        }
    }

    pub fn is_done(&self) -> bool {
        return self.progress == self.total;
    }

    pub fn is_started(&self) -> bool {
        return self.started_at == None;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDomain {
    pub action_id: ActionID,
    pub priority: u64,
    pub stage: CommandStage,
    pub created_at: Timestamp,
    pub ttl: Timestamp,
    #[serde(flatten)]
    pub action: Action,
    pub progress: Progression,
}

impl CommandDomain {
    fn status(&self) -> String {
        let s = match self.stage {
            CommandStage::Completed => "completed",
            CommandStage::InProgress => {
                if self.progress.is_started() {
                    "running"
                } else {
                    "in_queue"
                }
            }
        };
        String::from(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "data", rename_all = "lowercase")]
pub enum Action {
    Inference(InferenceArgs),
    Network(NetworkArgs),
    Compute(ComputeArgs),
}
