use crate::web_utils::invoke_parse_log_error;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[derive(Deserialize)]
struct GetInsetResponse {
    inset: f64,
}

async fn get_top_inset() -> Option<GetInsetResponse> {
    invoke_parse_log_error::<Option<GetInsetResponse>>(
        "plugin:safe-area-insets-css|get_top_inset",
        &Some(serde_json::json!({"paylaod": {}})),
    )
    .await
    .flatten()
}

async fn get_bottom_inset() -> Option<GetInsetResponse> {
    invoke_parse_log_error::<Option<GetInsetResponse>>(
        "plugin:safe-area-insets-css|get_bottom_inset",
        &Some(serde_json::json!({"paylaod": {}})),
    )
    .await
    .flatten()
}

pub async fn set_inset() {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    if let Some(inset) = get_top_inset().await
        && let Some(doc_element) = document.document_element()
        && let Ok(element) = doc_element.dyn_into::<web_sys::HtmlElement>()
    {
        element
            .style()
            .set_property("--safe-area-inset-top", &format!("{}px", inset.inset))
            .unwrap();
    }
    if let Some(inset) = get_bottom_inset().await
        && let Some(doc_element) = document.document_element()
        && let Ok(element) = doc_element.dyn_into::<web_sys::HtmlElement>()
    {
        element
            .style()
            .set_property("--safe-area-inset-bottom", &format!("{}px", inset.inset))
            .unwrap();
    }
}
