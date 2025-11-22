use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    pub on_close: Callback<()>,
}

#[function_component(AddArticleModal)]
pub fn add_article_modal(props: &ModalProps) -> Html {
    let url_input = use_state(|| String::new());

    let on_url_change = {
        let url_input = url_input.clone();
        Callback::from(move |e: InputEvent| {
            let target: web_sys::HtmlInputElement = e.target_unchecked_into();
            url_input.set(target.value());
        })
    };

    let paste_from_clipboard = {
        let url_input = url_input.clone();
        Callback::from(move |_| {
            let url_input = url_input.clone();
            spawn_local(async move {
                if let Some(window) = window() {
                    let promise = window.navigator().clipboard().read_text();
                    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
                    if let Ok(text) = result {
                        url_input.set(text.as_string().unwrap_or_default());
                    }
                }
            });
        })
    };

    let on_submit = {
        let url_input = url_input.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let url = (*url_input).clone();
            let on_close = on_close.clone();

            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({"url": url})).unwrap();
                invoke("add_article", args).await;
                on_close.emit(());
            });
        })
    };

    let on_overlay_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    let stop_propagation = Callback::from(|e: MouseEvent| e.stop_propagation());

    html! {
        <div class="modal-overlay" onclick={on_overlay_click}>
            <div class="modal-content" onclick={stop_propagation}>
                <h2>{"Add New Article"}</h2>
                <form onsubmit={on_submit}>
                    <label>
                        {"Article URL"}
                        <input
                            type="url"
                            value={(*url_input).clone()}
                            oninput={on_url_change}
                            placeholder="https://example.com/article"
                            required=true
                        />
                    </label>

                    <button type="button" onclick={paste_from_clipboard} class="secondary">
                        <i class="ti ti-clipboard"></i> {" Paste from Clipboard"}
                    </button>

                    <div style="display: flex; gap: 1rem; margin-top: 1rem;">
                        <button type="submit">{"Add Article"}</button>
                        <button type="button" onclick={props.on_close.reform(|_| ())} class="secondary">
                            {"Cancel"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
