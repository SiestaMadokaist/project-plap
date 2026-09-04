use pkg::displayable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthorizedRoute {
    #[serde(rename = "/models/list")]
    ListModels,
    #[serde(rename = "/models/preview")]
    ModelPreview,
    #[serde(rename = "/agents/command/cp")]
    AgentModelCP,
    #[serde(rename = "/agents/command/list")]
    CommandList,
    #[serde(rename = "/agents/command/delete")]
    CommandDelete,
    // #[serde(rename = "/agents/command/delete")]
    // AgentCommandDelete,
    // #[serde(rename = "/agents/command/generate")]
    // AgentCommandGenerate,
    #[serde(rename = "/hq/instance/launch")]
    HQInstanceLaunch,
    #[serde(rename = "/hq/instance/control")]
    HQInstanceControl,
    #[serde(rename = "/hq/instance/list")]
    HQInstanceList,
    #[serde(rename = "/story/templates")]
    TemplateList,
    #[serde(rename = "/story/templates/create")]
    TemplateWrite,
    #[serde(rename = "/story/templates/delete")]
    TemplateDelete,
    #[serde(rename = "/story/templates/read")]
    TemplateRead,
}
displayable!(AuthorizedRoute);
