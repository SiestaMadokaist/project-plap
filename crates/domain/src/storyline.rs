use pkg::id_type;
use serde::{Deserialize, Serialize};

use crate::{
    commands::inference::{InferenceConfig, Inferrable},
    storage::StoragePath,
};

id_type!(StoryTemplateId);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawTemplate(String);
impl Inferrable for RawTemplate {}

use std::collections::{HashMap, HashSet};

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
 * data structure stored in S3 as file.
 * directly targeted by filename.
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Storyline {
    id: StoryTemplateId,
    inferences: Vec<InferenceConfig<RawTemplate>>,
    variables: HashMap<String, String>,
}

impl Storyline {
    pub fn id(&self) -> &StoryTemplateId {
        &self.id
    }

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
    use pkg::utils::testhelper::{self};

    use crate::storyline::Storyline;

    #[test]
    fn shape_test() -> Result<(), testhelper::Error> {
        let story = testhelper::read_json::<Storyline>(concat!(
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
