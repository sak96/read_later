use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(
        js_namespace = ["window", "__TAURI__", "clipboardManager"],
        js_name = readText
    )]
    async fn read_clipboard() -> JsValue;
}

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    pub open: bool,
    pub on_close: Callback<()>,
}

#[function_component(AddArticleModal)]
pub fn add_article_modal(props: &ModalProps) -> Html {
    let url_input = use_state(String::new);

    let on_url_change = {
        let url_input = url_input.clone();
        Callback::from(move |e: InputEvent| {
            let target: web_sys::HtmlInputElement = e.target_unchecked_into();
            url_input.set(target.value());
        })
    };
    let on_close = {
        let url_input = url_input.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |_: ()| {
            url_input.set(String::new());
            on_close.emit(());
        })
    };

    let paste_from_clipboard = {
        let url_input = url_input.clone();
        Callback::from(move |_| {
            let url_input = url_input.clone();
            spawn_local(async move {
                let result = read_clipboard().await;
                if let Ok(text) = serde_wasm_bindgen::from_value::<String>(result) {
                    url_input.set(text.as_str().to_string());
                }
            });
        })
    };

    let on_submit = {
        let url_input = url_input.clone();
        let on_close = on_close.clone();
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

    html! {
        <dialog open={props.open}>
            <article>
                <header>
                    <button aria-label="Close" rel="prev" onclick={on_close.reform(|_| ())}></button>
                </header>
                <footer>
                    <form onsubmit={on_submit}>
                        <input
                            type="url"
                            value={(*url_input).clone()}
                            oninput={on_url_change}
                            placeholder="https://example.com/article"
                            required=true
                        />
                        <div style="display: flex; gap: 1rem; margin-top: 1rem;">

                            <button type="button" onclick={paste_from_clipboard} class="secondary">
                                <i class="ti ti-clipboard"></i>
                            </button>
                            <button type="submit">
                                <i class="ti ti-check"></i>
                            </button>
                        </div>
                    </form>
                </footer>
            </article>
        </dialog>
    }
}
