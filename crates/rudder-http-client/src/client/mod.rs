#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use anyhow::{Result, bail};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};

mod cloudflare;

use self::cloudflare::CloudflareClient;

#[derive(Debug, Clone)]
pub struct Client {
    headers: HeaderMap,
}

impl Client {
    #[must_use]
    pub fn new() -> Self {
        let headers = HeaderMap::from_iter([
            (CONTENT_TYPE, HeaderValue::from_static("application/json")),
            (ACCEPT, HeaderValue::from_static("application/json")),
            (
                USER_AGENT,
                HeaderValue::from_static(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                )),
            ),
        ]);
        Self { headers }
    }

    pub fn cloudflare(&self, api_token: impl AsRef<str>) -> Result<CloudflareClient> {
        let api_token = api_token.as_ref().trim();
        if api_token.is_empty() {
            bail!("invalid api credentials: api token is empty")
        }

        let api_token = format!("Bearer {api_token}").into();

        let inner = reqwest::Client::builder()
            .default_headers(self.headers.clone())
            .build()
            .unwrap();

        Ok(CloudflareClient { inner, api_token })
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
