use pkg::displayable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthorizedRoute {
    #[serde(rename = "/models/list")]
    ListModels,
    #[serde(rename = "/agents/command/cp")]
    AgentModelCP,
    // #[serde(rename = "/agents/command/delete")]
    // AgentCommandDelete,
    // #[serde(rename = "/agents/command/generate")]
    // AgentCommandGenerate,
    // #[serde(rename = "/hq/instance/control")]
    // HQInstanceControl,
    #[serde(rename = "/story/templates")]
    TemplateList,
}
displayable!(AuthorizedRoute);
