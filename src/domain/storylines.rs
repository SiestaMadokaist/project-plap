use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::domain::{
    commands::inference::{InferenceConfig, Inferrable},
    storage::StoragePath,
};

pub struct PromptRequest {}

/**
 * the original unevaluated string
 * it still has comment, variables etc
 * @example
 * """
 * # Qualifiers
 * {{qualifiers}}
 *
 * # MC Female
 * {{mc:female}}
 *
 * # Expression
 * happy, smile, winking.
 * """
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Template(String);
impl Inferrable for Template {}

/**
 * data structure stored in S3 as file.
 * directly targeted by filename.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Storyline {
    name: String,
    inferences: Vec<InferenceConfig<Template>>,
    variables: HashMap<String, String>,
}

impl Storyline {
    pub fn loras(&self) -> HashSet<StoragePath> {
        let mut hash = HashSet::<StoragePath>::new();
        let inferences = &self.inferences;
        for inf in inferences.into_iter() {
            for lora in &inf.loras {
                hash.insert(lora.clone());
            }
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::storylines::Storyline,
        pkg::utils::testhelper::{self},
    };

    #[test]
    fn shape_test() -> Result<(), testhelper::Error> {
        let story =
            testhelper::read_json::<Storyline>("./samples/inputs/jsons/domain/storyline.json")?;
        let mc = story.variables.get("mc:male").unwrap();
        let expected_mc: String = "mizuki arknights".into();
        let line = story.inferences.first().unwrap();
        assert_eq!(mc, &expected_mc);
        assert_eq!(story.loras().len(), 2);
        assert_eq!(line.seed, -1);
        Ok(())
    }
}
