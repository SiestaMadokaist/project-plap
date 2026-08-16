use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    domain::commands::{
        command::Action::Network, compute::ComputeArgs, inference::InferenceArgs,
        network::NetworkArgs,
    },
    pkg::{
        macros::id_type,
        types::{
            time::{self, Second, Timestamp},
            unit::{self, Index0, Index1},
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
    Cancelled,
}

impl From<CommandStage> for String {
    fn from(value: CommandStage) -> Self {
        let s: &str = match value {
            CommandStage::InProgress => "in_progress",
            CommandStage::Completed => "completed",
            CommandStage::Cancelled => "cancelled",
        };
        String::from(s)
    }
}

id_type!(ActionId);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Progression {
    total: unit::Index0,
    progress: unit::Index0,
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
        return self.started_at != None;
    }
}

const DEFAULT_TTL_SECONDS: i64 = 86400;

fn default_ttl() -> Option<Timestamp> {
    Some(Timestamp(Timestamp::now().0 + DEFAULT_TTL_SECONDS))
}

fn deserialize_ttl<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
where
    D: Deserializer<'de>,
{
    let ttl = Option::<Timestamp>::deserialize(deserializer)?;
    Ok(ttl.or_else(default_ttl))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDomain {
    #[serde(default = "default_ttl", deserialize_with = "deserialize_ttl")]
    ttl: Option<Timestamp>,
    pub action_id: ActionId,
    pub priority: u64,
    pub stage: CommandStage,
    pub created_at: Timestamp,
    #[serde(flatten)]
    pub action: Action,
    pub progress: Progression,
}

impl CommandDomain {
    pub fn network(action_id: ActionId, args: NetworkArgs, priority: u64) -> Self {
        let now = Timestamp::now();
        let ttl = now.add(Second(86400));
        Self {
            ttl: Some(ttl),
            action_id,
            progress: Progression {
                total: Index0(1),
                progress: Index0(0),
                started_at: None,
                finished_at: None,
            },
            priority,
            stage: CommandStage::InProgress,
            action: Network(args),
            created_at: now,
        }
    }
    pub fn status(&self) -> String {
        let s = match self.stage {
            CommandStage::Completed => "completed",
            CommandStage::Cancelled => "cancelled",
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
