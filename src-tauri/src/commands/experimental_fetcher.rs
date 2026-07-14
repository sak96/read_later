//! Fetches article HTML by navigating the main webview to the external URL.
//!
//! Some articles require login, captcha, or JavaScript rendering that a plain HTTP
//! request cannot handle. This module navigates the app's main webview directly to
//! the article URL so the user can complete any such process in the live page.
//!
//! # Flow
//!
//! 1. [`ExperimentalFetcher::new`] captures the current browser history length,
//!    then navigates the webview to the target URL.
//! 2. [`ExperimentalFetcher::fetch`] injects a floating toolbar (OK / Cancel) into
//!    the external page. The user interacts with the page as needed (login, captcha,
//!    etc.) and clicks **OK** to capture the rendered HTML, or **Cancel** to abort.
//! 3. After capture, the caller parses and saves the article to the database.
//! 4. [`Drop`] navigates the webview back to the app by replaying browser history
//!    (`history.go(-(window.history.length - initial))`). This handles any number
//!    of intermediate pages the user may have visited (login redirects, etc.).
//!
//! An [`ArticleFetchLock`] (channel-based semaphore) in the caller ensures only one
//! external fetch runs at a time, preventing concurrent navigation of the webview.

use serde::Deserialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::Duration;
use tauri::webview::WebviewWindow;
use tauri::{AppHandle, Listener, Manager, Runtime};

const HTML_FETCHER: &str = "html-fetcher";
const HISTORY_LEN_EVENT: &str = "__experimental_fetcher_history_len";
const INJECTOR_INTERVAL: Duration = Duration::from_millis(500);
const HISTORY_LEN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
struct HtmlFetcherResponse {
    url: String,
    origin: String,
    path: String,
    html: Option<String>,
}

pub trait HtmlFetcher {
    fn fetch(&mut self) -> Result<String, String>;
}

pub struct ExperimentalFetcher<R: Runtime> {
    app: AppHandle<R>,
    webview: WebviewWindow<R>,
    url: String,
    self_origin: String,
    self_path: String,
    initial_history_length: u32,
}

impl<R: Runtime> ExperimentalFetcher<R> {
    pub fn new(app: &AppHandle<R>, url: &str) -> Result<Self, String> {
        let webview = app
            .get_webview_window("main")
            .ok_or("Failed to get main webview")?;

        let (hist_tx, hist_rx) = mpsc::channel::<u32>();
        let hist_listener = app.listen(HISTORY_LEN_EVENT, move |event| {
            if let Ok(len) = serde_json::from_str::<u32>(event.payload()) {
                let _ = hist_tx.send(len);
            }
        });

        webview
            .eval(format!(
                "window.__TAURI__.event.emit(\"{HISTORY_LEN_EVENT}\", window.history.length)"
            ))
            .map_err(|e| format!("Failed to get current window history: {e}"))?;

        let initial_history_length = hist_rx
            .recv_timeout(HISTORY_LEN_TIMEOUT)
            .map_err(|_| "Failed to get current window history".to_string())?;
        app.unlisten(hist_listener);

        let escaped_url = serde_json::to_string(url).map_err(|e| e.to_string())?;
        webview
            .eval(format!("window.location.href = {escaped_url};"))
            .map_err(|e| e.to_string())?;

        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;
        let self_origin = parsed.origin().ascii_serialization();
        let self_path = parsed.path().to_string();

        Ok(Self {
            app: app.clone(),
            webview,
            url: url.to_string(),
            self_origin,
            self_path,
            initial_history_length,
        })
    }
}

impl<R: Runtime> HtmlFetcher for ExperimentalFetcher<R> {
    fn fetch(&mut self) -> Result<String, String> {
        let (tx, rx) = mpsc::channel::<HtmlFetcherResponse>();
        let running = Arc::new(AtomicBool::new(false));

        running.store(true, Ordering::Relaxed);
        let running_clone = running.clone();

        let listener_id = self.app.listen(HTML_FETCHER, move |event| {
            let payload_str = event.payload().to_string();
            let parsed: Result<HtmlFetcherResponse, _> = serde_json::from_str(&payload_str);

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
            thread::sleep(INJECTOR_INTERVAL / 2);
            while injector_running.load(Ordering::Relaxed) {
                let _ = injector_webview.eval(
                    r##"
                    if (!document.getElementById("__tauri_capture_toolbar")) {
                        const bar = document.createElement("div");
                        bar.id = "__tauri_capture_toolbar";

                        Object.assign(bar.style, {
                            position: "fixed",
                           right: "20px",
                            bottom: "20px",
                            zIndex: "2147483647",
                            display: "flex",
                            flexDirection: "column",
                            gap: "5px"
                        });

                        const buttonStyle = {
                            margin: "0",
                            backgroundColor: "#0172ad",
                            color: "#eff1f4",
                            minWidth: "1.5em",
                            minHeight: "1.5em",
                            fontSize: "1.5em",
                            padding: "0.75rem 1.25rem",
                            border: "1px solid transparent",
                            borderRadius: "0.5rem",
                            cursor: "pointer",
                            fontWeight: "600",
                            textAlign: "center",
                            transition: "background-color .2s, border-color .2s, color .2s"
                        };

                        // OK button
                        const ok = document.createElement("button");
                        ok.textContent = "✓";
                        ok.className = "contrast";
                        Object.assign(ok.style, buttonStyle);

                        ok.onclick = () => {
                            document.getElementById("__tauri_capture_toolbar")?.remove();
                            window.__TAURI__.event.emit(
                                "html-fetcher",
                                {
                                    url: window.location.href,
                                    origin: window.location.origin,
                                    path: window.location.pathname,
                                    html: document.documentElement ? document.documentElement.outerHTML : null
                                }
                            );
                        };

                        // Cancel button
                        const cancel = document.createElement("button");
                        cancel.textContent = "×";
                        cancel.className = "secondary";
                        Object.assign(cancel.style, buttonStyle);

                        cancel.onclick = () => {
                            window.__TAURI__.event.emit("html-fetcher", {
                                url: window.location.href,
                                origin: window.location.origin,
                                path: window.location.pathname,
                                html: null
                            });
                        };

                        bar.appendChild(ok);
                        bar.appendChild(cancel);

                        if (document.body) {
                            document.body.appendChild(bar);
                        }
                    }
                    "##,
                );
                thread::sleep(INJECTOR_INTERVAL);
            }
        });

        let response = rx.recv().map_err(|e| e.to_string())?;
        let _ = injector_handle.join();
        self.app.unlisten(listener_id);
        let _ = self
            .webview
            .eval(r#"document.getElementById("__tauri_capture_toolbar")?.remove();"#);
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
}

impl<R: Runtime> Drop for ExperimentalFetcher<R> {
    fn drop(&mut self) {
        let _ = self.webview.eval(format!(
            "window.history.go(-(window.history.length - {})); setTimeout(() => window.location.reload(), {})",
            self.initial_history_length,
            INJECTOR_INTERVAL.as_millis()
        ));
    }
}
