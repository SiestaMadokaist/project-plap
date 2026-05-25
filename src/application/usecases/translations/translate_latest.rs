use std::rc::Rc;
use tokio::sync::OnceCell;

use crate::{
    application::{
        ports::{
            clients::{raws::RawsClient, translator::TranslatorClient},
            repository::translation::TranslationRepository,
        },
        usecases::errors::UsecaseError,
    },
    domain::translation::{ChapterId, NovelId, TranslationProgress},
};

pub struct TranslateLatest {
    pub novel_id: NovelId,
    pub chapter_id: ChapterId,
}

pub trait Repos {
    type TR: TranslationRepository;
    fn translation_repo(&self) -> &Self::TR;
}

pub trait Clients {
    type TC: TranslatorClient;
    type Raw: RawsClient;
    fn translator(&self) -> Self::TC;
    fn raw(&self) -> Self::Raw;
}

pub struct Params {
    id: NovelId,
}
struct Memo {
    latest_raw: OnceCell<ChapterId>,
    untranslated_chapters: OnceCell<Vec<ChapterId>>,
}
impl Memo {
    fn new() -> Self {
        return Memo {
            latest_raw: OnceCell::new(),
            untranslated_chapters: OnceCell::new(),
        };
    }
}

pub struct RunTranslation<R: Repos, C: Clients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: Params,
    memo: Memo,
}

impl<R: Repos, C: Clients> RunTranslation<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: Params) -> Self {
        return RunTranslation {
            repo,
            client,
            params,
            memo: Memo::new(),
        };
    }

    async fn latest_translation(&self) -> Result<Option<TranslationProgress>, UsecaseError> {
        let translation_repo = self.repo.translation_repo();
        let result = translation_repo.latest(&self.params.id).await?;
        return Ok(result);
    }

    async fn latest_raw(&self) -> Result<&ChapterId, UsecaseError> {
        let chapter = self
            .memo
            .latest_raw
            .get_or_try_init(async || {
                let ch = self.client.raw().latest(&self.params.id).await;
                return ch;
            })
            .await?;
        return Ok(chapter);
    }

    async fn untranslated(&self) -> Result<&Vec<ChapterId>, UsecaseError> {
        let result = self
            .memo
            .untranslated_chapters
            .get_or_try_init(|| async {
                let latest_raw = self.latest_raw().await?;
                let latest_translation = self.latest_translation().await?;
                let latest_translated_chapter = latest_translation
                    .as_ref()
                    .map(|x| x.latest_chapter())
                    .unwrap_or(&ChapterId(0));
                let iterator = latest_translated_chapter.until(latest_raw);
                return Ok::<Vec<ChapterId>, UsecaseError>(iterator);
            })
            .await?;
        return Ok(result);
    }

    pub async fn exec(&self) -> Result<(), UsecaseError> {
        let untranslated = self.untranslated().await?;
        let translation = self.repo.translation_repo();
        for chapter in untranslated {
            let raw = self.client.raw().read(chapter).await?;
            let translated = self.client.translator().translate(&raw).await?;
            // todo: save to s3,
            print!("{}", &translated);
            translation.set_latest(&self.params.id, chapter).await?;
        }
        return Ok(());
    }

    // async fn is_untranslated(&self) -> Result<bool, UsecaseError> {
    //     let latest_chapter  = self.latest_raw().await?;
    //     let in_db = self.in_db().await?;
    //     let result: bool = match in_db {
    //         None => true,
    //         Some(x) => x.is_untranslated(latest_chapter),
    //     };
    //     return Ok(result);
    // }

    // async fn fetch_raw(&self) -> Result<String, UsecaseError> {
    //     let latest = self.latest_raw().await?

    // }
}
