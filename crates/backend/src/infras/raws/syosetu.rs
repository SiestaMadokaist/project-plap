use reqwest::{header, Client};
use scraper::error::SelectorErrorKind;

use crate::application::ports::clients::raws::RawsClient;
use crate::infras::http_error::reqwest_error;
use domain::{
    errors::DomainError,
    translation::{ChapterId, NovelId},
};

pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub struct Syosetu {
    host: String,
    client: Client,
}

static SELECTOR: &str = ".p-novel__body";
impl Syosetu {
    pub fn new(host: String, proxy: Option<ProxyConfig>) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36"));
        let mut builder = Client::builder().default_headers(headers);
        if let Some(p) = proxy {
            let proxy_url = format!("http://{}:{}", p.host, p.port);
            let proxy = reqwest::Proxy::all(&proxy_url)
                .expect("invalid proxy url")
                .basic_auth(&p.username, &p.password);
            tracing::info!("proxy_url: {}, username: {}", proxy_url, p.username);
            builder = builder.proxy(proxy);
        }
        Syosetu {
            host,
            client: builder.build().expect("failed to build reqwest client"),
        }
    }

    pub async fn fetch(
        &self,
        novel_id: &NovelId,
        chapter: &ChapterId,
    ) -> Result<String, DomainError> {
        let url = format!("{}/{}/{}", self.host, novel_id.0, chapter.0);
        tracing::info!("url: {}", url);
        let body = self.client.get(&url).send().await.map_err(reqwest_error)?;
        let text = body.text().await.map_err(reqwest_error)?;
        tracing::info!("response: {}", text);
        let html = scraper::Html::parse_document(&text);
        let selector = scraper::Selector::parse(SELECTOR).map_err(selector_error)?;
        let content = html.select(&selector).next();
        // tracing::info!("content: {}", content.as_ref().map((c) => c.));
        let source: Result<String, DomainError> = match content {
            None => Err(DomainError::EmptyResponse),
            Some(s) => {
                let t = s.text();
                let col: Vec<&str> = t.collect();
                return Ok(col.join("\n"));
            }
        };
        source
    }
}

// `DomainError` now lives in the separate `domain` crate, so `impl From<X> for
// DomainError` here would violate the orphan rules (neither type is local to
// this crate). A plain conversion function used via `.map_err(..)` instead.
fn selector_error(e: SelectorErrorKind<'_>) -> DomainError {
    DomainError::InvalidSelector(e.to_string())
}

impl RawsClient for Syosetu {
    async fn latest(&self, novel_id: &NovelId) -> Result<ChapterId, DomainError> {
        let url = format!(
            "https://api.syosetu.com/novelapi/api/?of=ga&ncode={}&out=json",
            novel_id.0
        );
        let resp = self.client.get(&url).send().await.map_err(reqwest_error)?;
        let text = resp.text().await.map_err(reqwest_error)?;
        tracing::info!("raw response: {}", text);
        let json: serde_json::Value = serde_json::from_str(&text)?;
        let count = json[1]["general_all_no"]
            .as_i64()
            .ok_or(DomainError::MissingContent)?;
        Ok(ChapterId(count as i32))
    }

    async fn read(
        &self,
        novel_id: &NovelId,
        chapter_id: &ChapterId,
    ) -> Result<String, DomainError> {
        self.fetch(novel_id, chapter_id).await
    }
}
