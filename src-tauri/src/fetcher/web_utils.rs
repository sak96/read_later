use serde::Deserialize;
use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::Duration;
use tauri::webview::WebviewWindow;
use tauri::{AppHandle, Listener, Manager, Runtime};

pub(crate) const HTML_CAPTURE_EVENT: &str = "__experimental_fetcher_html_capture";
pub(crate) const HISTORY_LEN_EVENT: &str = "__experimental_fetcher_history_len";
pub(crate) const PAGE_LOADED_EVENT: &str = "__experimental_fetcher_page_loaded";
pub(crate) const PAGE_RELOAD_DONE_EVENT: &str = "__experimental_fetcher_reload_done";

pub(crate) const HISTORY_LEN_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const PAGE_LOAD_CHECK_INTERVAL: Duration = Duration::from_millis(50);
pub(crate) const PAGE_LOAD_MIN_ATTEMPTS: u32 = 5;
pub(crate) const PAGE_LOAD_MAX_ATTEMPTS: u32 = 100;
pub(crate) const PAGE_LOAD_POST_READY_DELAY: Duration = Duration::from_millis(500);
pub(crate) const PAGE_LOAD_INITIAL_DELAY: Duration = Duration::from_millis(150);

pub(crate) const TOOLBAR_REMOVE_JS: &str =
    r#"document.getElementById("__tauri_capture_toolbar_host")?.remove();"#;

pub(crate) const PAGE_READY_CHECK_JS: &str =
    r#"(document.readyState === "complete" || document.readyState === "interactive")"#;

#[derive(Deserialize)]
pub(crate) struct CaptureResponse {
    pub url: String,
    pub origin: String,
    pub path: String,
    pub html: Option<String>,
}

pub(crate) struct FetcherBase<R: Runtime> {
    pub app: AppHandle<R>,
    pub webview: WebviewWindow<R>,
    pub url: String,
    pub self_origin: String,
    pub self_path: String,
    pub initial_history_length: Option<u32>,
}

impl<R: Runtime> FetcherBase<R> {
    pub fn new(app: &AppHandle<R>, url: &str) -> Result<Self, String> {
        let webview = app
            .get_webview_window("main")
            .ok_or("Failed to get main webview")?;

        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;

        Ok(Self {
            app: app.clone(),
            webview,
            url: url.to_string(),
            self_origin: parsed.origin().ascii_serialization(),
            self_path: parsed.path().to_string(),
            initial_history_length: None,
        })
    }

    pub fn get_history(&self) -> Result<u32, String> {
        let (hist_tx, hist_rx) = mpsc::channel::<u32>();
        let hist_listener = self.app.listen(HISTORY_LEN_EVENT, move |event| {
            if let Ok(len) = serde_json::from_str::<u32>(event.payload()) {
                let _ = hist_tx.send(len);
            }
        });

        self.webview
            .eval(format!(
                r#"window.__TAURI__.event.emit("{event_name}", window.history.length)"#,
                event_name = HISTORY_LEN_EVENT,
            ))
            .map_err(|e| format!("Failed to get current window history: {e}"))?;

        let history_length = hist_rx
            .recv_timeout(HISTORY_LEN_TIMEOUT)
            .map_err(|_| "Failed to get current window history".to_string())?;
        self.app.unlisten(hist_listener);

        Ok(history_length)
    }

    pub fn wait_for_page_ready(&self, event_name: &str) -> Result<(), String> {
        let webview_clone = self.webview.clone();
        let (tx, rx) = mpsc::channel::<()>();
        let event = event_name.to_string();
        let event_clone = event.clone();

        let listener = self.app.listen(event, move |_| {
            let _ = tx.send(());
        });

        thread::spawn(move || {
            let mut attempts = 0u32;
            thread::sleep(PAGE_LOAD_INITIAL_DELAY);
            loop {
                thread::sleep(PAGE_LOAD_CHECK_INTERVAL);
                attempts += 1;

                let eval_result = webview_clone.eval(PAGE_READY_CHECK_JS);
                if eval_result.is_ok() && attempts > PAGE_LOAD_MIN_ATTEMPTS {
                    thread::sleep(PAGE_LOAD_POST_READY_DELAY);
                    Self::emit_event(&webview_clone, &event_clone);
                    break;
                }
                if attempts > PAGE_LOAD_MAX_ATTEMPTS {
                    Self::emit_event(&webview_clone, &event_clone);
                    break;
                }
            }
        });

        rx.recv_timeout(HISTORY_LEN_TIMEOUT * 2)
            .map_err(|_| "Timed out waiting for page to load".to_string())?;
        self.app.unlisten(listener);

        Ok(())
    }

    pub fn navigate_to_url(&self, url: &str) -> Result<(), String> {
        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;
        self.webview
            .navigate(parsed.clone())
            .map_err(|e| format!("Failed to navigate: {e}"))?;

        thread::sleep(PAGE_LOAD_INITIAL_DELAY);
        let mut attempts = 0u32;
        loop {
            thread::sleep(PAGE_LOAD_CHECK_INTERVAL);
            attempts += 1;

            if let Ok(current) = self.webview.url() {
                if current.origin() == parsed.origin()
                    && current.path().trim_end_matches('/') == parsed.path().trim_end_matches('/')
                {
                    break;
                }
            }

            if attempts > PAGE_LOAD_MAX_ATTEMPTS {
                return Err("Timed out waiting for navigation URL to match".into());
            }
        }

        self.wait_for_page_ready(PAGE_LOADED_EVENT)
    }

    pub fn listen_for_capture(
        &self,
        running: Arc<AtomicBool>,
    ) -> (tauri::EventId, mpsc::Receiver<CaptureResponse>) {
        let (tx, rx) = mpsc::channel();
        let running_clone = running.clone();
        let listener_id = self.app.listen(HTML_CAPTURE_EVENT, move |event| {
            match serde_json::from_str::<CaptureResponse>(event.payload()) {
                Ok(response) => {
                    let _ = tx.send(response);
                }
                Err(err) => {
                    eprintln!("Failed to parse html-fetcher event: {err}");
                }
            }
            running_clone.store(false, Ordering::Relaxed);
        });
        (listener_id, rx)
    }

    pub fn validate_response(&self, response: CaptureResponse) -> Result<String, String> {
        match response.html {
            None => Err("Cancelled by user".into()),
            Some(html) if html.is_empty() => Err("Page returned empty content".into()),
            Some(_)
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

    pub fn navigate_back_if_needed(&mut self) {
        if let Some(initial) = self.initial_history_length {
            let _ = self.webview.eval(format!(
                "window.history.go(-(window.history.length - {initial}));",
                initial = initial,
            ));
            let _ = self.wait_for_page_ready(PAGE_RELOAD_DONE_EVENT);
        }
    }

    pub fn emit_event(webview: &WebviewWindow<impl Runtime>, event_name: &str) {
        let _ = webview.eval(format!(
            r#"window.__TAURI__.event.emit("{event_name}");"#,
            event_name = event_name,
        ));
    }
}

pub(crate) struct ToolbarInjector<R: Runtime> {
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    _phantom: PhantomData<R>,
}

impl<R: Runtime> ToolbarInjector<R> {
    pub fn spawn(webview: WebviewWindow<R>, running: Arc<AtomicBool>, bottom_inset: f64) -> Self {
        let handle = {
            let running = running.clone();
            thread::spawn(move || {
                thread::sleep(INJECTOR_INTERVAL / 2);
                let iframe_html = build_iframe_html();
                while running.load(Ordering::Relaxed) {
                    let js = build_toolbar_inject_js(&iframe_html, bottom_inset);
                    let _ = webview.eval(&js);
                    thread::sleep(INJECTOR_INTERVAL);
                }
            })
        };
        Self {
            handle: Some(handle),
            running,
            _phantom: PhantomData,
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl<R: Runtime> Drop for ToolbarInjector<R> {
    fn drop(&mut self) {
        self.stop();
    }
}

pub(crate) struct FetchGuard<R: Runtime> {
    pub app: AppHandle<R>,
    pub webview: WebviewWindow<R>,
    pub listener_id: tauri::EventId,
    pub injector: Option<ToolbarInjector<R>>,
    pub remove_toolbar: bool,
}

impl<R: Runtime> Drop for FetchGuard<R> {
    fn drop(&mut self) {
        if let Some(mut inj) = self.injector.take() {
            inj.stop();
        }
        self.app.unlisten(self.listener_id);
        if self.remove_toolbar {
            let _ = self.webview.eval(TOOLBAR_REMOVE_JS);
        }
    }
}

const INJECTOR_INTERVAL: Duration = Duration::from_millis(500);

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
        <button class="btn" id="__tauri_cap_ok">✓</button>
        <button class="btn" id="__tauri_cap_cancel">×</button>
    </div>
"#;

fn build_iframe_html() -> String {
    format!(
        r#"<!DOCTYPE html><html><head><style>{style}</style></head><body>{buttons}<script>
            document.getElementById('__tauri_cap_ok').onclick = function() {{
                window.parent.__TAURI__.event.emit('{event}', {{
                    url: window.parent.location.href,
                    origin: window.parent.location.origin,
                    path: window.parent.location.pathname,
                    html: window.parent.document.documentElement?.outerHTML ?? null
                }});
            }};
            document.getElementById('__tauri_cap_cancel').onclick = function() {{
                window.parent.__TAURI__.event.emit('{event}', {{
                    url: window.parent.location.href,
                    origin: window.parent.location.origin,
                    path: window.parent.location.pathname,
                    html: null
                }});
            }};
        </script></body></html>"#,
        style = IFRAME_STYLE,
        buttons = IFRAME_BUTTONS_HTML,
        event = HTML_CAPTURE_EVENT,
    )
}

fn build_toolbar_inject_js(iframe_html: &str, bottom_inset: f64) -> String {
    let html = serde_json::to_string(iframe_html).unwrap_or_default();
    format!(
        r#"
            if (!document.getElementById("__tauri_capture_toolbar_host")) {{
                const iframe = document.createElement("iframe");
                iframe.id = "__tauri_capture_toolbar_host";
                iframe.style.cssText = "position:fixed !important;right:20px !important;bottom: {bottom}px !important;z-index:2147483647 !important;border:none !important;width:60px !important;height:auto !important;pointer-events:auto !important;background:transparent !important;";
                (document.documentElement || document.body).appendChild(iframe);
                iframe.scrollIntoView({{behavior: "smooth", block: "center"}});
                iframe.contentDocument.open();
                iframe.contentDocument.write({html});
                iframe.contentDocument.close();
            }}
        "#,
        bottom = bottom_inset,
        html = html,
    )
}
