use serde::{Deserialize, Serialize};

use crate::{displayable, pkg::enums::stage::Stage};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum DDBTable {
    #[serde(rename = "agent_commands")]
    AgentCommands,
    #[serde(rename = "hot_reloads")]
    HotReloads,
    #[serde(rename = "translations")]
    Translations,
}
displayable!(DDBTable);

impl DDBTable {
    pub fn table_name(&self, stage: Stage) -> String {
        let v: Vec<String> = vec![stage.into(), self.into()];
        v.join("-")
    }
}
