use std::rc::Rc;

use aws_config::SdkConfig;

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
    pub fn rc(config: &SdkConfig, stage: Stage) -> Rc<Self> {
        Rc::new(Self::new(config, stage))
    }

    pub fn new(config: &SdkConfig, stage: Stage) -> Self {
        let client = aws_sdk_dynamodb::Client::new(config);
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
