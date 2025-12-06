use crate::web_utils::{invoke_no_parse_log_error, invoke_parse_log_error, transform_callback};
use shared::models::IntentEvent;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub async fn add_share_listener(callback: Callback<String>) -> u32 {
    // on open check if there is anything in queue.
    if let Some(IntentEvent::TextIntent(url)) = invoke_parse_log_error::<IntentEvent>(
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
            if let Some(IntentEvent::TextIntent(url)) = invoke_parse_log_error::<IntentEvent>(
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
