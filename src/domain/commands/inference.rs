use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::{
    domain::storage::StoragePath,
    pkg::types::{strings::URL, unit},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowId(pub String);

pub trait Inferrable {}
/**
 * the evaluated string
 * comments is stripped
 * variables is evaluated
 *
 * the evaluation is performed is in the frontend side
 * @example
 * """
 * # Qualifiers
 * masterpiece, best quality, absurdres
 *
 * # MC Female
 * lumine from genshin impact
 *
 * # Expression
 * happy, smile, winking.
 * """
 */
impl Inferrable for String {}
/**
 * when the prompts request already evaluated and ready to be queued
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceConfig<S: Inferrable> {
    workflow_id: Option<WorkflowId>,
    loras: Vec<StoragePath>,
    positive: S,
    negative: S,
    width: unit::Px,
    height: unit::Px,
    steps: unit::Index1,
    /// how many image generated per request
    #[serde(deserialize_with = "deserialize_n_iter")]
    n_iter: unit::Index1,
    seed: u32,
    /// workflow will iterate for each node
    /// if a node that consume reference, the next node
    /// that need reference will use the next reference
    /// eg: if node 3 use reference it use references[0]
    /// then if node 5 also need reference, it'll use references[1]
    references: Vec<URL>,
}

fn deserialize_n_iter<'de, D>(deserializer: D) -> Result<unit::Index1, D::Error>
where
    D: Deserializer<'de>,
{
    let n_iter = unit::Index1::deserialize(deserializer)?;
    // for now limited to 5 because each image can take upto 12 seconds to generate.
    // and the agent sending request to comfyui is waiting the whole batch to complete.
    // which will timed out and not considered task completed.
    if n_iter.0 >= 5 {
        return Err(D::Error::custom("n_iter must be less than 5"));
    }
    Ok(n_iter)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceArgs {
    pub config: InferenceConfig<String>,
}

#[cfg(test)]
mod tests {
    use crate::domain::commands::inference::InferenceConfig;

    #[test]
    fn shape_test() -> () {
        let buffer = std::fs::read("./samples/inputs/jsons/domain/commands/inference.json")
            .expect("cannot find inference.json");
        let cfg: InferenceConfig<String> = serde_json::from_slice(&buffer)
            .expect("cannot deserialize buffer to InferenceConfig<String>");
        assert_eq!(cfg.height.0, 1000);
        assert_eq!(cfg.width.0, 800);
        assert!(cfg.positive.starts_with("(( highly detailed, photograph"));
        assert!(cfg.positive.ends_with("\nADDCOMM"));
        assert_eq!(cfg.loras.len(), 2);
        let workflow_id = cfg.workflow_id.expect("workflow is null");
        assert_eq!(workflow_id.0, "test-wf-1");
    }
}
