use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlAnchorElement, js_sys, window};
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], js_name = "transformCallback")]
    pub fn transform_callback(handler: &js_sys::Function, once: bool) -> u32;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);

    #[wasm_bindgen(thread_local_v2, js_namespace = ["window", "__TAURI_OS_PLUGIN_INTERNALS__"], js_name = "os_type")]
    static OS_TYPE: String;

}

pub async fn sleep(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = window().expect("no global `window` exists");
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .expect("should register timeout");
    });
    if let Err(err) = wasm_bindgen_futures::JsFuture::from(promise).await {
        error(&format!("sleep failed with err: {:?}", err.as_string()));
    };
}

pub fn is_android() -> bool {
    OS_TYPE.with(|os_type| os_type == "android")
}

pub async fn invoke_no_parse(
    cmd: &str,
    args: &Option<serde_json::Value>,
) -> Result<JsValue, String> {
    let args = match args {
        Some(args) => serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?,
        None => JsValue::NULL,
    };
    invoke(cmd, args).await.map_err(|err| {
        err.as_string()
            .unwrap_or_else(|| "Unknown Error".to_owned())
    })
}

pub async fn invoke_no_parse_log_error(cmd: &str, args: &Option<serde_json::Value>) {
    if let Err(err) = invoke_no_parse(cmd, args).await {
        error(&format!("`{cmd}` failed with error: {err}"));
    }
}

pub async fn invoke_parse<T>(cmd: &str, args: &Option<serde_json::Value>) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let result = invoke_no_parse(cmd, args).await?;
    serde_wasm_bindgen::from_value::<T>(result).map_err(|e| e.to_string())
}

pub async fn invoke_parse_log_error<T>(cmd: &str, args: &Option<serde_json::Value>) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    invoke_parse(cmd, args)
        .await
        .map_err(|err| error(&format!("`{cmd}` failed with error: {err}")))
        .ok()
}

pub async fn read_clipboard() -> Option<String> {
    invoke_parse::<String>("plugin:clipboard-manager|read_text", &None)
        .await
        .ok()
}

pub async fn open_url(url: String) {
    invoke_no_parse_log_error(
        "plugin:opener|open_url",
        &Some(serde_json::json!({"url": url})),
    )
    .await
}

pub fn extract_text(id: usize) -> Option<String> {
    let window = window()?;
    let document = window.document()?;
    document
        .get_element_by_id(&format!("para_{}", id))?
        .text_content()
}

pub fn scroll_to_top(element_id: usize) {
    scroll_to_element(element_id, web_sys::ScrollLogicalPosition::Start);
}
pub fn scroll_to_center(element_id: usize) {
    scroll_to_element(element_id, web_sys::ScrollLogicalPosition::Center);
}

fn scroll_to_element(element_id: usize, position: web_sys::ScrollLogicalPosition) {
    if let Some(window) = window()
        && let Some(document) = window.document()
        && let Some(element) = document.get_element_by_id(&format!("para_{}", element_id))
    {
        let scroll_into_view_options = web_sys::ScrollIntoViewOptions::new();
        scroll_into_view_options.set_behavior(web_sys::ScrollBehavior::Smooth);
        scroll_into_view_options.set_block(position);
        element.scroll_into_view_with_scroll_into_view_options(&scroll_into_view_options);
    }
}

// TTS plugin connectors
pub async fn speak(text: String, rate: f32) {
    invoke_no_parse_log_error(
        "plugin:tts|speak",
        &Some(serde_json::json!({"args": {"text": text, "rate": rate}})),
    )
    .await;
}

pub async fn stop_speak() {
    invoke_no_parse_log_error("plugin:tts|stop", &None).await;
}

pub fn find_visible_para_id() -> usize {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let window_height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);
    let mut id = 0;
    while let Some(element) = document.get_element_by_id(&format!("para_{}", id)) {
        let rect = element.get_bounding_client_rect();
        if rect.bottom() >= 0.0 && rect.bottom() <= window_height {
            return id;
        }
        id += 1;
    }
    id - 1
}

pub async fn add_share_listener(callback: Callback<String>) -> u32 {
    // on open check if there is anything in queue.
    if let Some(url) = invoke_parse_log_error::<String>(
        "plugin:mobile-sharetarget|pop_intent_queue_and_extract_text",
        &None,
    )
    .await
    {
        callback.emit(url.to_string());
    }
    // then register a callback
    let closure = Closure::wrap(Box::new(move |_| {
        let callback = callback.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(url) = invoke_parse_log_error::<String>(
                "plugin:mobile-sharetarget|pop_intent_queue_and_extract_text",
                &None,
            )
            .await
            {
                callback.emit(url.to_string());
            }
        });
    }) as Box<dyn FnMut(JsValue)>);
    let id = transform_callback(closure.as_ref().unchecked_ref::<_>(), false);
    invoke_no_parse_log_error(
        "plugin:event|listen",
        &Some(
            serde_json::json!({"event":"tauri://focus", "handler": id, "target": {"kind": "Any"}}),
        ),
    )
    .await;
    closure.forget();
    id
}

pub async fn remove_share_listener(id: u32) {
    invoke_no_parse_log_error(
        "plugin:sharetarget|remove_listener",
        &Some(serde_json::json!({"event":"share", "channelId": id})),
    )
    .await
}

pub fn set_callback_to_link(div: &NodeRef, on_click: Callback<String>, url: String) {
    if let Some(div) = div.cast::<Element>() {
        let anchors = div.query_selector_all("a").unwrap();
        for i in 0..anchors.length() {
            if let Some(anchor) = anchors.item(i) {
                let anchor = anchor.dyn_into::<HtmlAnchorElement>().unwrap();
                let mut href = anchor.href();
                if href.starts_with('#') {
                    href = format!("{url}{href}");
                }
                let on_click = on_click.clone();
                let closure = Closure::wrap(Box::new(move || {
                    on_click.emit(href.clone());
                }) as Box<dyn FnMut()>);
                anchor
                    .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
                anchor.set_href("javascript:void(0)");
            }
        }
    }
}

pub async fn get_version() -> Option<String> {
    invoke_parse_log_error::<String>("plugin:app|version", &None).await
}
