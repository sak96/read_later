use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlAnchorElement, window};
use yew::prelude::*;

pub fn extract_text(div: &NodeRef, id: usize) -> Option<String> {
    if let Some(div) = div.cast::<Element>()
        && let Ok(Some(element)) = div.query_selector(&format!(".tts_para_{id}"))
    {
        element.text_content()
    } else {
        None
    }
}

pub fn find_visible_para_id() -> usize {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let window_height = window
        .inner_height()
        .ok()
        .and_then(|x| x.as_f64())
        .unwrap_or(0.0);
    let mut id = 0;
    while let Ok(Some(element)) = document.query_selector(&format!(".tts_para_{id}")) {
        let rect = element.get_bounding_client_rect();
        if rect.bottom() >= 0.0 && rect.top() <= window_height {
            return id;
        }
        id += 1;
    }
    id - 1
}

pub fn set_callback_to_link(div: &NodeRef, on_click: Callback<String>, id: i32) {
    let page_url = format!("http://tauri.localhost/article/{id}#");
    if let Some(div) = div.cast::<Element>() {
        let anchors = if let Ok(anchors) = div.query_selector_all("a") {
            anchors
        } else {
            return;
        };
        for i in 0..anchors.length() {
            if let Some(anchor) = anchors.item(i)
                && let Ok(anchor) = anchor.dyn_into::<HtmlAnchorElement>()
            {
                let href = anchor.href();
                if href.starts_with(&page_url) {
                    continue;
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
