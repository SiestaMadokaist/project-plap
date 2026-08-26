use std::rc::Rc;

use aws_sdk_dynamodb::Client;

use backend::{
    application::ports::repository::container::{HasHotReload, HasTranslation},
    constant::ddb::DDBTable,
    infras::repos::dynamo::{
        hotreload::DDBHotReloadRepository, translation::DDBTranslationRepository,
    },
};
use pkg::enums::stage::Stage;

pub struct CronRepos {
    translation: DDBTranslationRepository,
    hotreload: DDBHotReloadRepository,
}

impl CronRepos {
    pub fn rc(client: &Client, stage: Stage) -> Rc<Self> {
        Rc::new(Self::new(client, stage))
    }

    pub fn new(client: &Client, stage: Stage) -> Self {
        Self {
            translation: DDBTranslationRepository::new(
                client.clone(),
                DDBTable::Translations.table_name(stage),
            ),
            hotreload: DDBHotReloadRepository::new(
                client.clone(),
                DDBTable::HotReloads.table_name(stage),
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
