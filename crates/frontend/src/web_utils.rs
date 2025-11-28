use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::js_sys;
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
