use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    commands::inference::{InferenceConfig, Inferrable},
    storage::StoragePath,
};
use pkg::macros::id_type;

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

id_type!(StorylineId);

/**
 * data structure stored in S3 as file.
 * directly targeted by filename.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Storyline {
    name: StorylineId,
    inferences: Vec<InferenceConfig<Template>>,
    variables: HashMap<String, String>,
}

impl Storyline {
    pub fn loras(&self) -> HashSet<StoragePath> {
        let mut hash = HashSet::<StoragePath>::new();
        let inferences = &self.inferences;
        for inf in inferences.iter() {
            for lora in &inf.loras {
                hash.insert(lora.clone());
            }
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use crate::storylines::Storyline;
    use pkg::utils::testhelper::{self};

    #[test]
    fn shape_test() -> Result<(), testhelper::Error> {
        let story =
            testhelper::read_json::<Storyline>(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../samples/inputs/jsons/domain/storyline.json"
            ))?;
        let mc = story.variables.get("mc:male").unwrap();
        let expected1: String = "mizuki from arknights".into();
        let mc2 = story.variables.get("mc:female").unwrap();
        let expected2: String = "w from arknights".into();
        let line = story.inferences.first().unwrap();
        assert_eq!(mc, &expected1);
        assert_eq!(mc2, &expected2);
        assert_eq!(story.loras().len(), 2);
        assert_eq!(line.seed, -1);
        Ok(())
    }
}
