use anyhow::{Context, Error, Result};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tauri::{AppHandle, Runtime};

use super::Fetcher;
use super::web_utils::{FetchGuard, FetcherBase, PAGE_LOAD_CHECK_INTERVAL};

const HTML_CAPTURE_JS: &str = r"
    window.__TAURI__.event.emit('__experimental_fetcher_html_capture', {
        url: window.location.href,
        origin: window.location.origin,
        path: window.location.pathname,
        html: document.documentElement.outerHTML
    });
";

pub struct HtmlJsFetcher<R: Runtime> {
    base: FetcherBase<R>,
}

impl<R: Runtime> HtmlJsFetcher<R> {
    pub fn new(app: &AppHandle<R>, url: &str) -> Result<Self, Error> {
        Ok(Self {
            base: FetcherBase::new(app, url).context("failed to create fetcher base")?,
        })
    }

    fn fetch_inner(&mut self) -> Result<String, Error> {
        self.base.remember_history();

        self.base
            .navigate_to_url(&self.base.url)
            .with_context(|| format!("failed to navigate to {}", self.base.url))?;

        let running = Arc::new(AtomicBool::new(true));

        let (listener_id, rx) = self.base.listen_for_capture(&running);

        self.base
            .webview
            .eval(HTML_CAPTURE_JS)
            .context("failed to evaluate HTML capture script")?;

        let response = rx
            .recv_timeout(PAGE_LOAD_CHECK_INTERVAL * 2)
            .context("timed out waiting for page HTML capture")?;

        let guard = FetchGuard {
            app: self.base.app.clone(),
            webview: self.base.webview.clone(),
            listener_id,
            injector: None,
            remove_toolbar: false,
        };

        drop(guard);

        self.base
            .validate_response(response)
            .context("failed to validate captured HTML response")
    }
}

impl<R: Runtime> Fetcher for HtmlJsFetcher<R> {
    fn fetch(&mut self) -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send + '_>> {
        Box::pin(async { self.fetch_inner() })
    }
}

impl<R: Runtime> Drop for HtmlJsFetcher<R> {
    fn drop(&mut self) {
        self.base.navigate_back_if_needed();
    }
}
