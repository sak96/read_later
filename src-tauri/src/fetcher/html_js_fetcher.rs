use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Runtime};

use super::Fetcher;
use super::web_utils::{FetchGuard, FetcherBase, HISTORY_LEN_TIMEOUT};

const HTML_CAPTURE_JS: &str = r#"
    window.__TAURI__.event.emit('__experimental_fetcher_html_capture', {
        url: window.location.href,
        origin: window.location.origin,
        path: window.location.pathname,
        html: document.documentElement.outerHTML
    });
"#;

pub struct HtmlJsFetcher<R: Runtime> {
    base: FetcherBase<R>,
}

impl<R: Runtime> HtmlJsFetcher<R> {
    pub fn new(app: &AppHandle<R>, url: &str) -> Result<Self, String> {
        Ok(Self {
            base: FetcherBase::new(app, url)?,
        })
    }

    fn fetch_inner(&mut self) -> Result<String, String> {
        self.base.initial_history_length = Some(self.base.get_history()?);
        self.base.navigate_to_url(&self.base.url)?;

        let running = Arc::new(AtomicBool::new(true));
        let (listener_id, rx) = self.base.listen_for_capture(running);

        let _ = self.base.webview.eval(HTML_CAPTURE_JS);
        let response = rx
            .recv_timeout(HISTORY_LEN_TIMEOUT * 2)
            .map_err(|_| "Timed out waiting for page capture".to_string())?;

        let _guard = FetchGuard {
            app: self.base.app.clone(),
            webview: self.base.webview.clone(),
            listener_id,
            injector: None,
            remove_toolbar: false,
        };
        drop(_guard);

        self.base.validate_response(response)
    }
}

impl<R: Runtime> Fetcher for HtmlJsFetcher<R> {
    fn fetch(&mut self) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        Box::pin(async { self.fetch_inner() })
    }
}

impl<R: Runtime> Drop for HtmlJsFetcher<R> {
    fn drop(&mut self) {
        self.base.navigate_back_if_needed();
    }
}
