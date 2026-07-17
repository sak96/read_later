use std::pin::Pin;
use std::future::Future;

use tauri_plugin_http::reqwest;

use super::Fetcher;

const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

pub struct HtmlFetcher {
    url: String,
}

impl HtmlFetcher {
    pub fn new(url: &str) -> Result<Self, String> {
        Ok(Self {
            url: url.to_string(),
        })
    }
}

impl Fetcher for HtmlFetcher {
    fn fetch(&mut self) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let url = self.url.clone();
        Box::pin(async move {
            reqwest::Client::new()
                .get(&url)
                .header(reqwest::header::USER_AGENT, CHROME_USER_AGENT)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())
        })
    }
}
