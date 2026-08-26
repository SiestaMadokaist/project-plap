use serde::{Deserialize, Deserializer, Serialize};

use crate::commands::{
    command::Action::Network, compute::ComputeArgs, inference::InferenceArgs,
    network::NetworkArgs,
};
use pkg::{
    macros::{displayable, id_type},
    types::{
        time::{Second, Timestamp},
        unit::{self, Index0, INDEX_ZERO},
    },
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
// #[serde(rename_all = "lowercase")]
pub enum CommandStage {
    #[serde(rename = "in_progress")]
    InProgress,
    // Running is skipped
    // we use InProgress { started_at: Some } to mark its Running
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "failed")]
    Failed,
}
displayable!(CommandStage);
id_type!(ActionId);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Progression {
    total: unit::Index0,
    progress: unit::Index0,
    finished_at: Option<Timestamp>,
    failed_at: Option<Timestamp>,
}

impl Progression {
    pub fn new(total: unit::Index0, progress: unit::Index0) -> Self {
        Self {
            total,
            progress,
            finished_at: None,
            failed_at: None,
        }
    }

    pub fn start(&mut self) {}

    pub fn fail(&mut self) {
        self.failed_at = Some(Timestamp::now());
    }

    pub fn increment(&mut self) {
        let now = Timestamp::now();
        self.progress.next();
        if self.is_done() {
            self.finished_at = Some(now);
        }
    }

    pub fn is_failed(&self) -> bool {
        self.failed_at.is_none() == false
    }

    pub fn is_done(&self) -> bool {
        self.failed_at.is_none() && self.progress == self.total
    }

    pub fn is_started(&self) -> bool {
        self.progress.gt(&INDEX_ZERO)
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
                finished_at: None,
                failed_at: None,
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
            CommandStage::Failed => "failed",
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

#[cfg(test)]
mod tests {
    use pkg::types::unit;

    use super::Progression;

    #[test]
    fn test_increment() -> std::io::Result<()> {
        let progress = &mut Progression::new(unit::Index0(10), unit::Index0(0));
        progress.increment();
        assert_eq!(progress.is_done(), false);
        Ok(())
    }

    #[test]
    fn test_done() -> std::io::Result<()> {
        let progress = &mut Progression::new(unit::Index0(1), unit::Index0(0));
        progress.increment();
        assert_eq!(progress.is_done(), true);
        Ok(())
    }
}
