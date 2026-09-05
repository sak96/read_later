use anyhow::{Context, Error, Result};
use std::future::Future;
use std::pin::Pin;

use tauri_plugin_http::reqwest;

use super::Fetcher;

const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

pub struct HtmlFetcher {
    url: String,
}

impl HtmlFetcher {
    pub fn new(url: &str) -> Result<Self, Error> {
        Ok(Self {
            url: url.to_string(),
        })
    }
}

impl Fetcher for HtmlFetcher {
    fn fetch(&mut self) -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send + '_>> {
        let url = self.url.clone();
        Box::pin(async move {
            let response = reqwest::Client::new()
                .get(&url)
                .header(reqwest::header::USER_AGENT, CHROME_USER_AGENT)
                .send()
                .await
                .with_context(|| format!("failed to send request to {url}"))?;
            let text = response
                .text()
                .await
                .with_context(|| format!("failed to read text response from {url}"))?;
            Ok(text)
        })
    }
}
