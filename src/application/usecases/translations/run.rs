use std::rc::Rc;
use tokio::sync::OnceCell;

use crate::{
    application::{
        dto::void::VoidDTO,
        ports::{
            clients::{
                notification::NotificationClient, raws::RawsClient, storage::StorageClient,
                translator::TranslatorClient,
            },
            repository::translation::TranslationRepository,
        },
        usecases::bases::Usecase,
    },
    domain::{
        errors::DomainError,
        translation::{ChapterId, NovelId, TranslationDomain},
    },
};

pub trait Repos {
    type TR: TranslationRepository;
    fn translation_repo(&self) -> &Self::TR;
}

pub trait Clients {
    type TC: TranslatorClient;
    type Raw: RawsClient;
    type Storage: StorageClient;
    type Notification: NotificationClient;
    fn translator(&self) -> &Self::TC;
    fn raw(&self) -> &Self::Raw;
    fn storage(&self) -> &Self::Storage;
    fn notification(&self) -> &Self::Notification;
}

pub struct Params {
    novel_id: NovelId,
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

pub struct Run<R: Repos, C: Clients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: Params,
    memo: Memo,
}

impl<R: Repos, C: Clients> Run<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: Params) -> Self {
        return Run {
            repo,
            client,
            params,
            memo: Memo::new(),
        };
    }

    async fn latest_translation(&self) -> Result<Option<TranslationDomain>, DomainError> {
        let translation_repo = self.repo.translation_repo();
        let result = translation_repo.latest(&self.params.novel_id).await?;
        return Ok(result);
    }

    async fn latest_raw(&self) -> Result<&ChapterId, DomainError> {
        let chapter = self
            .memo
            .latest_raw
            .get_or_try_init(async || {
                let ch = self.client.raw().latest(&self.params.novel_id).await;
                return ch;
            })
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
                let iterator = latest_translated_chapter.until(latest_raw);
                return Ok::<Vec<ChapterId>, DomainError>(iterator.collect());
            })
            .await?;
        return Ok(result);
    }
}

impl<R: Repos, C: Clients> Usecase<()> for Run<R, C> {
    type Output = VoidDTO;
    async fn exec(self) -> Result<VoidDTO, DomainError> {
        let untranslated = self.untranslated().await?;
        let tl_repo = self.repo.translation_repo();
        let storage = self.client.storage();
        let notification = self.client.notification();
        for chapter_id in untranslated {
            let raw = self
                .client
                .raw()
                .read(&self.params.novel_id, chapter_id)
                .await?;
            let translated = self.client.translator().translate(&raw).await?;
            let inserted = tl_repo.insert(&self.params.novel_id, chapter_id).await?;
            // let progress = tl_repo.set_latest(&self.params.novel_id, chapter).await?;
            storage
                .write(inserted.filepath(), translated.bytes())
                .await?;
            let notification_message = format!(
                "chapter {} of {:?} has just been translated",
                inserted.title(),
                inserted.chapter_id()
            );
            notification.notify(&notification_message).await?;
        }
        return Ok(VoidDTO {});
    }
}
