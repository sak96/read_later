use std::pin::Pin;
use std::future::Future;

use tauri::AppHandle;
use tauri::Wry;

mod web_utils;
mod html_fetcher;
mod html_js_fetcher;
mod html_js_auth_fetcher;

pub use html_fetcher::HtmlFetcher;
pub use html_js_fetcher::HtmlJsFetcher;
pub use html_js_auth_fetcher::HtmlJsAuthFetcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetcherMode {
    Html,
    HtmlJs,
    HtmlJsAuth,
}

impl FetcherMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "html" => Some(Self::Html),
            "html_js" => Some(Self::HtmlJs),
            "html_js_auth" => Some(Self::HtmlJsAuth),
            _ => None,
        }
    }
}

pub trait Fetcher: Send {
    fn fetch(&mut self) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;
}

pub fn new_fetcher(app: &AppHandle<Wry>, url: &str, mode: FetcherMode) -> Result<Box<dyn Fetcher>, String> {
    match mode {
        FetcherMode::Html => Ok(Box::new(HtmlFetcher::new(url)?)),
        FetcherMode::HtmlJs => Ok(Box::new(HtmlJsFetcher::new(app, url)?)),
        FetcherMode::HtmlJsAuth => Ok(Box::new(HtmlJsAuthFetcher::new(app, url)?)),
    }
}
