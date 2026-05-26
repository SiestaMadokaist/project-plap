use reqwest::Client;
use scraper::error::SelectorErrorKind;

use crate::domain::errors::DomainError;
// pub struct Host(String);

pub struct Syosetu {
    host: String,
    client: Client,
}

static SELECTOR: &'static str = ".p-novel__body";
impl Syosetu {
    pub fn new(host: String) -> Self {
        Syosetu {
            host,
            client: Client::new(),
        }
    }

    pub async fn fetch(&self, novel_id: &str, chapter: i16) -> Result<String, DomainError> {
        let url = format!("{}/{}/{}", self.host, novel_id, chapter);
        let body = self.client.get(&url).send().await?;
        let text = body.text().await?;
        let html = scraper::Html::parse_document(&text);
        let selector = scraper::Selector::parse(SELECTOR)?;
        let content = html.select(&selector).next();
        let source: Result<String, DomainError> = match content {
            None => Err(DomainError::EmptyResponse),
            Some(s) => {
                let t = s.text();
                let col: Vec<&str> = t.collect();
                return Ok(col.join("\n"));
            }
        };
        return source;
    }
}

impl From<reqwest::Error> for DomainError {
    fn from(e: reqwest::Error) -> DomainError {
        if e.is_connect() || e.is_timeout() {
            DomainError::HttpConnectionFailed(e.to_string())
        } else {
            DomainError::HttpError(e.to_string())
        }
    }
}

impl From<SelectorErrorKind<'_>> for DomainError {
    fn from(e: SelectorErrorKind<'_>) -> DomainError {
        DomainError::InvalidSelector(e.to_string())
    }
}
