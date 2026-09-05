use anyhow::{Context, Error, Result};
use readabilityrs::Readability;
use std::future::Future;
use std::pin::Pin;

use tauri::AppHandle;
use tauri::Wry;

mod html_fetcher;
mod html_js_auth_fetcher;
mod html_js_fetcher;
mod web_utils;

pub use html_fetcher::HtmlFetcher;
pub use html_js_auth_fetcher::HtmlJsAuthFetcher;
pub use html_js_fetcher::HtmlJsFetcher;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FetcherMode {
    #[default]
    Html,
    HtmlJs,
    HtmlJsAuth,
}

impl FromStr for FetcherMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(Self::Html),
            "html_js" => Ok(Self::HtmlJs),
            "html_js_auth" => Ok(Self::HtmlJsAuth),
            _ => Err(()),
        }
    }
}

pub trait Fetcher: Send {
    fn fetch(&mut self) -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send + '_>>;
}

pub fn new_fetcher(
    app: &AppHandle<Wry>,
    url: &str,
    mode: FetcherMode,
) -> Result<Box<dyn Fetcher>, Error> {
    match mode {
        FetcherMode::Html => Ok(Box::new(HtmlFetcher::new(url)?)),
        FetcherMode::HtmlJs => Ok(Box::new(HtmlJsFetcher::new(app, url)?)),
        FetcherMode::HtmlJsAuth => Ok(Box::new(HtmlJsAuthFetcher::new(app, url)?)),
    }
}

pub async fn fetch_parse_update_article(
    article_url: &str,
    fetcher: &mut dyn Fetcher,
) -> Result<(String, String, String), Error> {
    let html = fetcher.fetch().await?;

    let options = readabilityrs::ReadabilityOptions::builder()
        .remove_title_from_content(true)
        .build();
    let article_data = Readability::new(&html, Some(article_url), Some(options))
        .with_context(|| format!("Failed to create readability for {article_url}"))?
        .parse()
        .context("Failed to parse article")?;

    let title = match article_data.title {
        Some(v) if v.is_empty() => "Untitled".into(),
        None => "Untitled".into(),
        Some(v) => v,
    };
    let body = article_data.content.unwrap_or_default();
    let text_content = article_data.text_content.unwrap_or_default();

    Ok((title, body, text_content))
}
