use std::rc::Rc;

use aws_sdk_dynamodb::Client;
use serde::{Deserialize, Serialize};

use crate::{
    application::ports::repository::container::{HasHotReload, HasTranslation},
    infras::repos::dynamo::{
        hotreload::DDBHotReloadRepository, translation::DDBTranslationRepository,
    },
    pkg::{enums::stage::Stage, macros::displayable},
};

pub struct CronRepos {
    translation: DDBTranslationRepository,
    hotreload: DDBHotReloadRepository,
}

/**
 * @todo:
 * this introduce a name change for table
 */
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TableName {
    Translations,
    HotReloads,
}
displayable!(TableName);

impl CronRepos {
    pub fn rc(client: &Client, stage: Stage) -> Rc<Self> {
        Rc::new(Self::new(client, stage))
    }

    fn to_table(stage: Stage, name: TableName) -> String {
        let v: Vec<String> = vec![stage.into(), name.into()];
        v.join("-")
    }

    pub fn new(client: &Client, stage: Stage) -> Self {
        Self {
            translation: DDBTranslationRepository::new(
                client.clone(),
                Self::to_table(stage, TableName::Translations),
            ),
            hotreload: DDBHotReloadRepository::new(
                client.clone(),
                Self::to_table(stage, TableName::HotReloads),
            ),
        }
    }
}

impl HasTranslation for CronRepos {
    type Translation = DDBTranslationRepository;
    fn translation(&self) -> &Self::Translation {
        &self.translation
    }
}

impl HasHotReload for CronRepos {
    type HotReload = DDBHotReloadRepository;
    fn hotreload(&self) -> &Self::HotReload {
        &self.hotreload
    }
}
