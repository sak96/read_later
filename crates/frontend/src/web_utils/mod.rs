use web_sys::{js_sys, window};
mod core;
mod dom;
mod scroll;
mod settings;
mod share;
mod speak;

use crate::web_utils::core::*;
pub use crate::web_utils::core::{
    get_version, invoke_no_parse, invoke_no_parse_log_error, invoke_parse, invoke_parse_log_error,
    is_android,
};
pub use crate::web_utils::dom::*;
pub use crate::web_utils::scroll::*;
pub use crate::web_utils::settings::*;
pub use crate::web_utils::share::*;
pub use crate::web_utils::speak::*;

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
