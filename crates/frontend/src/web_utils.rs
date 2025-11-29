use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCollection, js_sys, window};
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;

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

pub fn extract_text(id: &str) -> Option<String> {
    let window = window()?;
    let document = window.document()?;
    document.get_element_by_id(id)?.text_content()
}

pub fn scroll_to_element(element_id: &str) {
    if let Some(window) = window()
        && let Some(document) = window.document()
        && let Some(element) = document.get_element_by_id(element_id)
    {
        element.scroll_into_view();
    }
}

pub async fn speak(text: String, rate: f32, language: String) {
    let length = text.chars().count();
    web_sys::console::log_3(&language.into(), &rate.into(), &text.into());
    // mock speak by sleeping
    let promise = web_sys::js_sys::Promise::new(&mut |yes, _| {
        let win = window().unwrap();
        win.set_timeout_with_callback_and_timeout_and_arguments_0(&yes, length as i32 * 60)
            .unwrap();
    });
    let js_fut = wasm_bindgen_futures::JsFuture::from(promise);
    let _ = js_fut.await;
}

pub async fn stop_speak() {
    web_sys::console::log_1(&"stopped_speaking".into());
}

pub fn find_visible_para_id() -> Option<usize> {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let x = window.inner_width().ok()?.as_f64()? / 2.0;
    let y = 10.0;
    let top_element = document.element_from_point(x as f32, y as f32)?;
    let mut queue = VecDeque::new();
    queue.push_back(top_element);
    while let Some(current) = queue.pop_front() {
        let id = current.id();
        if id.starts_with("para_") {
            return id
                .strip_prefix("para_")
                .and_then(|s| s.parse::<usize>().ok());
        }
        let children: HtmlCollection = current.children();
        let length = children.length();

        for i in 0..length {
            if let Some(child) = children.item(i) {
                queue.push_back(child);
            }
        }
    }
    None
}
