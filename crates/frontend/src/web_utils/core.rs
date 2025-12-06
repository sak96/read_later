use wasm_bindgen::prelude::*;
use web_sys::js_sys;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], catch)]
    pub(super) async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], js_name = "transformCallback")]
    pub(super) fn transform_callback(handler: &js_sys::Function, once: bool) -> u32;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub(super) fn error(s: &str);

    #[wasm_bindgen(thread_local_v2, js_namespace = ["window", "__TAURI_OS_PLUGIN_INTERNALS__"], js_name = "os_type")]
    pub static OS_TYPE: String;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], js_name = "unregisterCallback")]
    pub(super) fn unregister_channel(id: u32);
}

pub fn is_android() -> bool {
    OS_TYPE.with(|os_type| os_type == "android")
}

pub async fn invoke_parse<T>(cmd: &str, args: &Option<serde_json::Value>) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let result = invoke_no_parse(cmd, args).await?;
    serde_wasm_bindgen::from_value::<T>(result).map_err(|e| e.to_string())
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

pub async fn invoke_parse_log_error<T>(cmd: &str, args: &Option<serde_json::Value>) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    invoke_parse(cmd, args)
        .await
        .map_err(|err| error(&format!("`{cmd}` failed with error: {err}")))
        .ok()
}

pub async fn invoke_no_parse_log_error(cmd: &str, args: &Option<serde_json::Value>) {
    if let Err(err) = invoke_no_parse(cmd, args).await {
        error(&format!("`{cmd}` failed with error: {err}"));
    }
}

pub async fn get_version() -> Option<String> {
    invoke_parse_log_error::<String>("plugin:app|version", &None).await
}
