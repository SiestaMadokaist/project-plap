use std::rc::Rc;
use tokio::sync::OnceCell;

use crate::{
    application::{
        dto::void::VoidDTO,
        ports::{
            clients::{
                cc::{self, AllClient},
                notification::NotificationClient,
                raws::RawsClient,
                storage::StorageClient,
                translator::TranslatorClient,
            },
            repository::{
                rc::{self, AllRepos},
                translation::TranslationRepository,
            },
        },
        usecases::bases::Usecase,
    },
    domain::{
        errors::DomainError,
        translation::{ChapterId, NovelId, TranslationDomain},
    },
};

pub trait Repos: rc::HasTranslation {}
impl<T: AllRepos> Repos for T {}

pub trait Clients: cc::HasTranslator + cc::HasRaws + cc::HasStorage + cc::HasNotification {}
impl<T: AllClient> Clients for T {}

pub struct Params {
    pub novel_id: NovelId,
}

struct Memo {
    latest_raw: OnceCell<ChapterId>,
    untranslated_chapters: OnceCell<Vec<ChapterId>>,
}

impl Memo {
    fn new() -> Self {
        Memo {
            latest_raw: OnceCell::new(),
            untranslated_chapters: OnceCell::new(),
        }
    }
}

pub struct Run<R: Repos, C: Clients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: Params,
    memo: Memo,
}

impl<R: Repos, C: Clients> Run<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: Params) -> Self {
        Run {
            repo,
            client,
            params,
            memo: Memo::new(),
        }
    }

    async fn latest_translation(&self) -> Result<Option<TranslationDomain>, DomainError> {
        let result = self
            .repo
            .translation()
            .latest(&self.params.novel_id)
            .await?;
        return Ok(result);
    }

    async fn latest_raw(&self) -> Result<&ChapterId, DomainError> {
        let chapter = self
            .memo
            .latest_raw
            .get_or_try_init(async || self.client.raws().latest(&self.params.novel_id).await)
            .await?;
        return Ok(chapter);
    }

    async fn untranslated(&self) -> Result<&Vec<ChapterId>, DomainError> {
        let result = self
            .memo
            .untranslated_chapters
            .get_or_try_init(|| async {
                let latest_raw = self.latest_raw().await?;
                let latest_translation = self.latest_translation().await?;
                let latest_translated_chapter = latest_translation
                    .as_ref()
                    .map(|x| x.chapter_id())
                    .unwrap_or(&ChapterId(0));
                Ok::<Vec<ChapterId>, DomainError>(
                    latest_translated_chapter.until(latest_raw).collect(),
                )
            })
            .await?;
        return Ok(result);
    }

    async fn run_translation(&self, prev: &TranslationDomain) -> Result<VoidDTO, DomainError> {
        let untranslated = self.untranslated().await?;
        let tl_repo = self.repo.translation();
        let storage = self.client.storage();
        let notification = self.client.notification();
        for chapter_id in untranslated {
            let raw = self
                .client
                .raws()
                .read(&self.params.novel_id, chapter_id)
                .await?;
            let translated = self.client.translator().translate(&raw).await?;
            let inserted = tl_repo.insert(prev, chapter_id).await?;
            storage
                .write(inserted.filepath(), translated.bytes())
                .await?;
            let message = format!(
                "chapter {} of {:?} has just been translated",
                inserted.title(),
                inserted.chapter_id()
            );
            notification.notify(&message).await?;
        }
        return Ok(VoidDTO {});
    }
}

impl<R: Repos, C: Clients> Usecase<()> for Run<R, C> {
    type Output = VoidDTO;

    async fn exec(self) -> Result<VoidDTO, DomainError> {
        let latest = self.latest_translation().await?;
        match latest {
            None => Err(DomainError::Prerequisite("story not initialized".into())),
            Some(latest) => self.run_translation(&latest).await,
        }
    }
}
