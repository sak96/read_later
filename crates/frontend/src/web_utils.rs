use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, window};
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], js_name="invoke", catch)]
    pub async fn invoke_catch(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], js_name = "transformCallback")]
    pub fn transform_callback(handler: &js_sys::Function, once: bool) -> u32;

}

pub async fn read_clipboard() -> Option<String> {
    let result = invoke("plugin:clipboard-manager|read_text", JsValue::NULL).await;
    serde_wasm_bindgen::from_value::<String>(result).ok()
}

pub async fn open_url(url: String) {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({"url": url})).unwrap();
    invoke("plugin:opener|open_url", args).await;
}
#[wasm_bindgen(
    inline_js = r#"export function ostype() { return window.__TAURI_OS_PLUGIN_INTERNALS__.os_type}"#
)]
extern "C" {
    pub fn ostype() -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type PluginListener;

    #[wasm_bindgen(method)]
    pub async fn unregister(this: &PluginListener) -> JsValue;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShareEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message<T> {
    message: T,
    index: u32,
}

pub async fn add_share_listener(callback: Callback<ShareEvent>) -> u32 {
    let closure = Closure::wrap(Box::new(move |event: JsValue| {
        match JsValueSerdeExt::into_serde::<Message<ShareEvent>>(&event) {
            Ok(share_event) => {
                callback.emit(share_event.message);
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Failed to deserialize share event: {:?}", e).into(),
                );
            }
        }
    }) as Box<dyn FnMut(JsValue)>);

    let id = transform_callback(closure.as_ref().unchecked_ref::<_>(), false);
    let args = serde_wasm_bindgen::to_value(
        &serde_json::json!({"event":"share", "handler": &format!("__CHANNEL__:{id}")}),
    )
    .unwrap();
    invoke("plugin:sharetarget|register_listener", args).await;
    closure.forget();
    id
}

pub async fn remove_share_listener(id: u32) {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({"event":"share", "channelId": id}))
        .unwrap();
    invoke("plugin:sharetarget|remove_listener", args).await;
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

pub async fn speak(text: String, _rate: f32, _language: String) {
    // TODO: handle _language and _rate
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({"text": text})).unwrap();
    if let Err(error) = invoke_catch("plugin:tts|speak", args).await {
        web_sys::console::error_2(&"speech issue".into(), &error);
    }
}

pub async fn stop_speak() {
    invoke("plugin:tts|stop", JsValue::NULL).await;
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
