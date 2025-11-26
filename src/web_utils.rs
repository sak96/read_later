use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn read_clipboard() -> Option<String> {
    let result = invoke("plugin:clipboard-manager|read_text", JsValue::NULL).await;
    serde_wasm_bindgen::from_value::<String>(result).ok()
}

pub async fn open_url(url: String) {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({"url": url})).unwrap();
    invoke("plugin:opener|open_url", args).await;
}
