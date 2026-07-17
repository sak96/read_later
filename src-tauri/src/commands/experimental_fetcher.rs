//! Fetches article HTML by navigating the main webview to the external URL.
//!
//! Some articles require login, captcha, or JavaScript rendering that a plain HTTP
//! request cannot handle. This module navigates the app's main webview directly to
//! the article URL so the user can complete any such process in the live page.
//!
//! # Flow
//!
//! 1. [`ExperimentalFetcher::new`] stores the target URL without navigating.
//! 2. [`ExperimentalFetcher::fetch`] captures the current browser history length,
//!    navigates the webview to the target URL, waits for the page to load, then
//!    injects a floating toolbar (OK / Cancel) into the external page. The user
//!    interacts with the page as needed (login, captcha, etc.) and clicks **OK**
//!    to capture the rendered HTML, or **Cancel** to abort.
//! 3. After capture, the caller parses and saves the article to the database.
//! 4. [`Drop`] navigates the webview back to the app by replaying browser history
//!    (`history.go(-(window.history.length - initial))`). This handles any number
//!    of intermediate pages the user may have visited (login redirects, etc.).

use serde::Deserialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::Duration;
use tauri::webview::WebviewWindow;
use tauri::{AppHandle, Listener, Manager, Runtime};
#[cfg(target_os = "android")]
use tauri_plugin_safe_area_insets_css::SafeAreaInsetsCssExt;

#[derive(Deserialize)]
struct CaptureResponse {
    url: String,
    origin: String,
    path: String,
    html: Option<String>,
}

pub struct ExperimentalFetcher<R: Runtime> {
    app: AppHandle<R>,
    webview: WebviewWindow<R>,
    url: String,
    self_origin: String,
    self_path: String,
    initial_history_length: Option<u32>,
}

impl<R: Runtime> ExperimentalFetcher<R> {
    const HTML_CAPTURE_EVENT: &str = "__experimental_fetcher_html_capture";
    const HISTORY_LEN_EVENT: &str = "__experimental_fetcher_history_len";
    const PAGE_LOADED_EVENT: &str = "__experimental_fetcher_page_loaded";
    const PAGE_RELOAD_DONE_EVENT: &str = "__experimental_fetcher_reload_done";
    const INJECTOR_INTERVAL: Duration = Duration::from_millis(500);
    const HISTORY_LEN_TIMEOUT: Duration = Duration::from_secs(5);
    const PAGE_LOAD_CHECK_INTERVAL: Duration = Duration::from_millis(50);
    const PAGE_LOAD_MIN_ATTEMPTS: u32 = 5;
    const PAGE_LOAD_MAX_ATTEMPTS: u32 = 100;
    const PAGE_LOAD_POST_READY_DELAY: Duration = Duration::from_millis(500);

    const IFRAME_STYLE: &str = r#"
        body { margin: 0; padding: 4px; background: transparent; overflow: visible }
        .bar { display: flex; flex-direction: column; gap: 5px; font-family: sans-serif }
        .btn {
            margin: 0; background-color: #0172ad; color: #eff1f4;
            min-width: 1.5em; min-height: 1.5em; font-size: 1.5em;
            padding: 0.75rem 1.25rem; border: 1px solid transparent;
            border-radius: 0.5rem; cursor: pointer; font-weight: 600; text-align: center
        }
    "#;

    const IFRAME_BUTTONS_HTML: &str = r#"
        <div class="bar">
            <button class="btn" id="__tauri_cap_ok">\u{2713}</button>
            <button class="btn" id="__tauri_cap_cancel">\u{00d7}</button>
        </div>
    "#;

    const TOOLBAR_REMOVE_JS: &str =
        r#"document.getElementById("__tauri_capture_toolbar_host")?.remove();"#;

    const PAGE_READY_CHECK_JS: &str =
        r#"(document.readyState === "complete" || document.readyState === "interactive")"#;

    fn build_iframe_html() -> String {
        let script = format!(
            r#"
                document.getElementById('__tauri_cap_ok').onclick = function() {{
                    window.parent.__TAURI__.event.emit('{capture_event}', {{
                        url: window.parent.location.href,
                        origin: window.parent.location.origin,
                        path: window.parent.location.pathname,
                        html: window.parent.document.documentElement
                            ? window.parent.document.documentElement.outerHTML
                            : null
                    }});
                }};
                document.getElementById('__tauri_cap_cancel').onclick = function() {{
                    window.parent.__TAURI__.event.emit('{capture_event}', {{
                        url: window.parent.location.href,
                        origin: window.parent.location.origin,
                        path: window.parent.location.pathname,
                        html: null
                    }});
                }};
            "#,
            capture_event = Self::HTML_CAPTURE_EVENT,
        );
        format!(
            r#"<!DOCTYPE html><html><head><style>{style}</style></head><body>{buttons}<script>{script}</script></body></html>"#,
            style = Self::IFRAME_STYLE,
            buttons = Self::IFRAME_BUTTONS_HTML,
            script = script,
        )
    }

    fn escape_for_js_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    pub fn new(app: &AppHandle<R>, url: &str) -> Result<Self, String> {
        let webview = app
            .get_webview_window("main")
            .ok_or("Failed to get main webview")?;

        #[cfg(target_os = "android")]
        {
            app.safe_area_insets_css();
        }

        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;
        let self_origin = parsed.origin().ascii_serialization();
        let self_path = parsed.path().to_string();

        Ok(Self {
            app: app.clone(),
            webview,
            url: url.to_string(),
            self_origin,
            self_path,
            initial_history_length: None,
        })
    }

    pub fn fetch(&mut self) -> Result<String, String> {
        self.initial_history_length = Some(self.get_history()?);
        self.open_and_wait_for_page()?;

        #[cfg(target_os = "android")]
        let bottom_inset = self
            .app
            .safe_area_insets_css()
            .get_bottom_inset()
            .map(|r| r.inset)
            .unwrap_or(0.0);

        #[cfg(not(target_os = "android"))]
        let bottom_inset = 0.0;

        let (tx, rx) = mpsc::channel::<CaptureResponse>();
        let running = Arc::new(AtomicBool::new(false));

        running.store(true, Ordering::Relaxed);
        let running_clone = running.clone();

        let listener_id = self.app.listen(Self::HTML_CAPTURE_EVENT, move |event| {
            let payload_str = event.payload().to_string();
            let parsed: Result<CaptureResponse, _> = serde_json::from_str(&payload_str);

            match parsed {
                Ok(response) => {
                    let _ = tx.send(response);
                }
                Err(err) => {
                    eprintln!("Failed to parse html-fetcher event: {err}");
                }
            }
            running_clone.store(false, Ordering::Relaxed);
        });

        let injector_webview = self.webview.clone();
        let injector_running = running.clone();
        let injector_handle = thread::spawn(move || {
            thread::sleep(Self::INJECTOR_INTERVAL / 2);
            while injector_running.load(Ordering::Relaxed) {
                let escaped_html = Self::escape_for_js_string(&Self::build_iframe_html());
                let js = format!(
                    r#"
                        if (!document.getElementById("__tauri_capture_toolbar_host")) {{
                            const iframe = document.createElement("iframe");
                            iframe.id = "__tauri_capture_toolbar_host";
                            iframe.style.cssText = "position:fixed !important;right:20px !important;bottom:calc(20px + {bottom}px) !important;z-index:2147483647 !important;border:none !important;width:60px !important;height:auto !important;pointer-events:auto !important;";
                            (document.documentElement || document.body).appendChild(iframe);
                            iframe.scrollIntoView({behavior: "smooth", block: "center"});
                            iframe.contentDocument.open();
                            iframe.contentDocument.write("{html}");
                            iframe.contentDocument.close();
                        }}
                    "#,
                    bottom = bottom_inset,
                    html = escaped_html,
                );
                let _ = injector_webview.eval(&js);
                thread::sleep(Self::INJECTOR_INTERVAL);
            }
        });

        let response = rx.recv().map_err(|e| e.to_string())?;
        let _ = injector_handle.join();
        self.app.unlisten(listener_id);
        let _ = self.webview.eval(Self::TOOLBAR_REMOVE_JS);

        match response.html {
            None => Err("Cancelled by user".into()),
            Some(html) if html.is_empty() => Err("Page returned empty content".into()),
            Some(_html)
                if response.origin != self.self_origin
                    || response.path.trim_end_matches('/')
                        != self.self_path.trim_end_matches('/') =>
            {
                Err(format!(
                    "Page navigated from {} to {} during fetch",
                    self.url, response.url
                ))
            }
            Some(html) => Ok(html),
        }
    }

    fn get_history(&self) -> Result<u32, String> {
        let (hist_tx, hist_rx) = mpsc::channel::<u32>();
        let hist_listener = self.app.listen(Self::HISTORY_LEN_EVENT, move |event| {
            if let Ok(len) = serde_json::from_str::<u32>(event.payload()) {
                let _ = hist_tx.send(len);
            }
        });

        self.webview
            .eval(format!(
                r#"window.__TAURI__.event.emit("{event_name}", window.history.length)"#,
                event_name = Self::HISTORY_LEN_EVENT,
            ))
            .map_err(|e| format!("Failed to get current window history: {e}"))?;

        let history_length = hist_rx
            .recv_timeout(Self::HISTORY_LEN_TIMEOUT)
            .map_err(|_| "Failed to get current window history".to_string())?;
        self.app.unlisten(hist_listener);

        Ok(history_length)
    }

    fn wait_for_page_ready(&self, event_name: &str) -> Result<(), String> {
        let webview_clone = self.webview.clone();
        let (tx, rx) = mpsc::channel::<()>();
        let event = event_name.to_string();
        let event_clone = event.clone();

        let listener = self.app.listen(event, move |_| {
            let _ = tx.send(());
        });

        thread::spawn(move || {
            let mut attempts = 0u32;
            loop {
                thread::sleep(Self::PAGE_LOAD_CHECK_INTERVAL);
                attempts += 1;

                let eval_result = webview_clone.eval(Self::PAGE_READY_CHECK_JS);
                if eval_result.is_ok() && attempts > Self::PAGE_LOAD_MIN_ATTEMPTS {
                    thread::sleep(Self::PAGE_LOAD_POST_READY_DELAY);
                    Self::emit_event(&webview_clone, &event_clone);
                    break;
                }
                if attempts > Self::PAGE_LOAD_MAX_ATTEMPTS {
                    Self::emit_event(&webview_clone, &event_clone);
                    break;
                }
            }
        });

        rx.recv_timeout(Self::HISTORY_LEN_TIMEOUT * 2)
            .map_err(|_| "Timed out waiting for page to load".to_string())?;
        self.app.unlisten(listener);

        Ok(())
    }

    fn open_and_wait_for_page(&self) -> Result<(), String> {
        let escaped_url = serde_json::to_string(&self.url).map_err(|e| e.to_string())?;
        self.webview
            .eval(format!(
                "window.location.href = {escaped_url};",
                escaped_url = escaped_url,
            ))
            .map_err(|e| e.to_string())?;

        self.wait_for_page_ready(Self::PAGE_LOADED_EVENT)
    }

    fn emit_event(webview: &WebviewWindow<impl Runtime>, event_name: &str) {
        let _ = webview.eval(format!(
            r#"window.__TAURI__.event.emit("{event_name}");"#,
            event_name = event_name,
        ));
    }
}

impl<R: Runtime> Drop for ExperimentalFetcher<R> {
    fn drop(&mut self) {
        if let Some(initial) = self.initial_history_length {
            let _ = self.webview.eval(format!(
                "window.history.go(-(window.history.length - {initial}));",
                initial = initial,
            ));
            let _ = self.wait_for_page_ready(Self::PAGE_RELOAD_DONE_EVENT);
        }
    }
}
