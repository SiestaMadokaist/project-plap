use reqwest::Client;
use scraper::error::SelectorErrorKind;
// pub struct Host(String);

pub struct Syosetu {
    host: String,
    client: Client,
}
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Unknown  Error")]
    UnknownError,
}

static SELECTOR: &'static str = ".p-novel__body";
impl Syosetu {
    pub fn new(host: String) -> Self {
        Syosetu {
            host,
            client: Client::new(),
        }
    }

    pub async fn fetch(&self, novel_id: &str, chapter: i16) -> Result<String, FetchError> {
        let url = format!("{}/{}/{}", self.host, novel_id, chapter);
        let body = self.client.get(&url).send().await?;
        let text = body.text().await?;
        let html = scraper::Html::parse_document(&text);
        let selector = scraper::Selector::parse(SELECTOR)?;
        let content = html.select(&selector).next();
        let source: Result<String, FetchError> = match content {
            None => Err(FetchError::UnknownError),
            Some(s) => {
                let t = s.text();
                let col: Vec<&str> = t.collect();
                return Ok(col.join("\n"));
            }
        };
        return source;
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(_: reqwest::Error) -> FetchError {
        FetchError::UnknownError
    }
}

impl From<SelectorErrorKind<'_>> for FetchError {
    fn from(_: SelectorErrorKind<'_>) -> FetchError {
        FetchError::UnknownError
    }
}
