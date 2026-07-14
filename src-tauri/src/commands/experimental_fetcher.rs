use serde::Deserialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::Duration;
use tauri::webview::WebviewWindow;
use tauri::{AppHandle, EventId, Listener, Manager, Runtime};

const HTML_FETCHER: &str = "html-fetcher";
const HISTORY_LEN_EVENT: &str = "__experimental_fetcher_history_len";
const INJECTOR_INTERVAL: Duration = Duration::from_millis(500);
const HISTORY_LEN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
struct HtmlFetcherResponse {
    url: String,
    html: Option<String>,
}

pub trait HtmlFetcher {
    fn fetch(&mut self) -> Result<String, String>;
}

pub struct ExperimentalFetcher<R: Runtime> {
    app: AppHandle<R>,
    webview: WebviewWindow<R>,
    url: String,
    initial_history_length: u32,
    running: Arc<AtomicBool>,
    listener_id: Option<EventId>,
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

        webview
            .eval(format!(r#"window.location.href = "{}";"#, url))
            .map_err(|e| e.to_string())?;

        Ok(Self {
            app: app.clone(),
            webview,
            url: url.to_string(),
            initial_history_length,
            running: Arc::new(AtomicBool::new(false)),
            listener_id: None,
        })
    }
}

impl<R: Runtime> HtmlFetcher for ExperimentalFetcher<R> {
    fn fetch(&mut self) -> Result<String, String> {
        let (tx, rx) = mpsc::channel::<HtmlFetcherResponse>();

        self.running.store(true, Ordering::Relaxed);
        let running_clone = self.running.clone();

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
        self.listener_id = Some(listener_id);

        let injector_webview = self.webview.clone();
        let injector_running = self.running.clone();

        thread::spawn(move || {
            while injector_running.load(Ordering::Relaxed) {
                let _ = injector_webview.eval(
                    r##"
                    const bar = document.createElement("div");
                    bar.id = "__tauri_capture_toolbar";

                    Object.assign(bar.style, {
                        position: "fixed",
                        right: "20px",
                        bottom: "20px",
                        zIndex: "2147483647",
                        display: "flex",
                        flexDirection: "column",
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
                        window.__TAURI__.event.emit(
                            "html-fetcher",
                            {
                                url: window.location.href,
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
                            html: null
                        });
                    };

                    bar.appendChild(ok);
                    bar.appendChild(cancel);

                    if (document.body) {
                        document.body.appendChild(bar);
                    }
                    "##,
                );

                thread::sleep(INJECTOR_INTERVAL);
            }
        });

        let response = rx.recv().map_err(|e| e.to_string())?;

        self.running.store(false, Ordering::Relaxed);

        match response.html {
            None => Err("Cancelled by user".into()),
            Some(html) if html.is_empty() => Err("Page returned empty content".into()),
            Some(_html) if response.url != self.url => Err(format!(
                "Page navigated from {} to {} during fetch",
                self.url, response.url
            )),
            Some(html) => Ok(html),
        }
    }
}

impl<R: Runtime> Drop for ExperimentalFetcher<R> {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(id) = self.listener_id.take() {
            self.app.unlisten(id);
        }
        let _ = self.webview.eval(format!(
            "window.history.go(-(window.history.length - {}))",
            self.initial_history_length
        ));
    }
}
