use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(
        js_namespace = ["window", "__TAURI__", "clipboardManager"],
        js_name = readText
    )]
    pub async fn read_clipboard() -> JsValue;

    #[wasm_bindgen(
        js_namespace = ["window", "__TAURI__", "opener"],
        js_name = openUrl
    )]
    pub async fn open_url(url: JsValue) -> JsValue;
}
