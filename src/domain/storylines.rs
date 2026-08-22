use serde::{Deserialize, Serialize};

use crate::domain::commands::inference::{InferenceConfig, Inferrable};

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
pub struct Storyline {
    name: String,
    inferences: Vec<InferenceConfig<Template>>,
}
