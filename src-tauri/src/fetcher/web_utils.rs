use serde::Deserialize;
use std::marker::PhantomData;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::Duration;
use tauri::webview::WebviewWindow;
use tauri::{AppHandle, Listener, Manager, Runtime};

pub(crate) const HTML_CAPTURE_EVENT: &str = "__experimental_fetcher_html_capture";
pub(crate) const PAGE_LOADED_EVENT: &str = "__experimental_fetcher_page_loaded";

pub(crate) const PAGE_LOAD_CHECK_INTERVAL: Duration = Duration::from_millis(50);
pub(crate) const PAGE_LOAD_MAX_ATTEMPTS: u32 = 100;
pub(crate) const PAGE_LOAD_INITIAL_DELAY: Duration = Duration::from_millis(500);

pub(crate) const TOOLBAR_REMOVE_JS: &str =
    r#"document.getElementById("__tauri_capture_toolbar_host")?.remove();"#;

pub(crate) fn page_ready_check_js(event_name: &str) -> String {
    format!(
        r#"(function() {{
            if (document.readyState === "complete" || document.readyState === "interactive") {{
                window.__TAURI__.event.emit("{event}");
            }}
        }})()"#,
        event = event_name,
    )
}

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
    pub back_url: Option<String>,
    pub self_origin: String,
    pub self_path: String,
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
            back_url: None,
            self_origin: parsed.origin().ascii_serialization(),
            self_path: parsed.path().to_string(),
        })
    }

    pub fn remember_history(&mut self) {
        self.back_url = self.webview.url().map(|url| url.to_string()).ok();
    }

    pub fn wait_for_page_ready(&self, event_name: &str) -> Result<(), String> {
        let webview_clone = self.webview.clone();
        let page_ready = Arc::new(AtomicBool::new(false));
        let page_ready_thread = page_ready.clone();
        let page_ready_main = page_ready.clone();
        let check_js = page_ready_check_js(event_name);

        let listener = self.app.listen(event_name.to_string(), move |_| {
            page_ready_thread.store(true, Ordering::Release);
        });

        let handle = thread::spawn(move || {
            let mut attempts = 0u32;
            thread::sleep(PAGE_LOAD_INITIAL_DELAY);
            while !page_ready.load(Ordering::Acquire) {
                thread::sleep(PAGE_LOAD_CHECK_INTERVAL);
                attempts += 1;
                let _ = webview_clone.eval(&check_js);
                if attempts > PAGE_LOAD_MAX_ATTEMPTS {
                    break;
                }
            }
        });

        let _ = handle.join();
        self.app.unlisten(listener);
        if !page_ready_main.load(Ordering::Acquire) {
            return Err("Timed out waiting for page to load".to_string());
        }

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

            if let Ok(current) = self.webview.url()
                && current.origin() == parsed.origin()
                && current.path().trim_end_matches('/') == parsed.path().trim_end_matches('/')
            {
                break;
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
        if let Some(target_url) = self.back_url.take() {
            let event_name = "__GO_BACK__";

            let js = format!(
                r#"
                (() => {{
                    window.addEventListener("popstate", () => {{
                        window.__TAURI__.event.emit(
                            {event},
                            window.location.href
                        );
                    }}, {{ once: true }});
                    window.history.back();
                }})()
                "#,
                event = event_name,
            );
            loop {
                let (tx, rx) = mpsc::channel::<String>();

                let listener_id = self.app.listen(event_name, move |event| {
                    let _ = tx.send(event.payload().to_string());
                });

                let _ = self.webview.eval(&js);

                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(url) => {
                        self.app.unlisten(listener_id);
                        if url == target_url {
                            break;
                        }
                    }

                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        self.app.unlisten(listener_id);
                        break;
                    }

                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        self.app.unlisten(listener_id);
                        break;
                    }
                }
            }
        }
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
