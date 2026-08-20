use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Runtime};
#[cfg(target_os = "android")]
use tauri_plugin_safe_area_insets_css::SafeAreaInsetsCssExt;

use super::Fetcher;
use super::web_utils::{FetchGuard, FetcherBase, ToolbarInjector};

pub struct HtmlJsAuthFetcher<R: Runtime> {
    base: FetcherBase<R>,
}

impl<R: Runtime> HtmlJsAuthFetcher<R> {
    pub fn new(app: &AppHandle<R>, url: &str) -> Result<Self, String> {
        Ok(Self {
            base: FetcherBase::new(app, url)?,
        })
    }

    fn fetch_inner(&mut self) -> Result<String, String> {
        self.base.remember_history();
        self.base.navigate_to_url(&self.base.url)?;

        #[cfg(target_os = "android")]
        let bottom_inset = self
            .base
            .app
            .safe_area_insets_css()
            .get_bottom_inset()
            .map(|r| r.inset)
            .unwrap_or(0.0);

        #[cfg(not(target_os = "android"))]
        let bottom_inset = 0.0;

        let running = Arc::new(AtomicBool::new(true));
        let (listener_id, rx) = self.base.listen_for_capture(&running);
        let injector = ToolbarInjector::spawn(self.base.webview.clone(), running, bottom_inset);

        let guard = FetchGuard {
            app: self.base.app.clone(),
            webview: self.base.webview.clone(),
            listener_id,
            injector: Some(injector),
            remove_toolbar: true,
        };

        let response = rx.recv().map_err(|e| e.to_string())?;
        drop(guard);

        self.base.validate_response(response)
    }
}

impl<R: Runtime> Fetcher for HtmlJsAuthFetcher<R> {
    fn fetch(&mut self) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        Box::pin(async { self.fetch_inner() })
    }
}

impl<R: Runtime> Drop for HtmlJsAuthFetcher<R> {
    fn drop(&mut self) {
        self.base.navigate_back_if_needed();
    }
}
