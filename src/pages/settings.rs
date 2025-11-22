use crate::routes::Route;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(Settings)]
pub fn settings() -> Html {
    let theme = use_state(|| "system".to_string());

    let theme_clone = theme.clone();
    use_effect_with((), move |_| {
        spawn_local(async move {
            let result = invoke(
                "get_setting",
                serde_wasm_bindgen::to_value(&serde_json::json!({"name": "theme"})).unwrap(),
            )
            .await;
            if let Ok(value) = serde_wasm_bindgen::from_value::<String>(result) {
                theme_clone.set(value);
            }
        });
    });

    let navigator = use_navigator().unwrap();
    let go_back = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });

    let on_theme_change = {
        let theme = theme.clone();
        Callback::from(move |e: Event| {
            let target: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = target.value();
            theme.set(value.clone());

            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "name": "theme",
                    "value": value
                }))
                .unwrap();
                invoke("set_setting", args).await;
            });
        })
    };

    html! {
        <>
            <nav class="container-fluid">
                <ul>
                    <li><button onclick={go_back} class="secondary">
                        <i class="ti ti-arrow-left"></i>
                    </button></li>
                    <li><strong>{"⚙️ Settings"}</strong></li>
                </ul>
            </nav>

            <main class="container">
                <article>
                    <h2>{"Appearance"}</h2>

                    <label>
                        {"Theme"}
                        <select value={(*theme).clone()} onchange={on_theme_change}>
                            <option value="light">{"Light"}</option>
                            <option value="dark">{"Dark"}</option>
                            <option value="system">{"System"}</option>
                        </select>
                    </label>
                </article>

                <article>
                    <h2>{"About"}</h2>
                    <p>
                        <a href="https://github.com" target="_blank">
                            <i class="ti ti-brand-github"></i> {" View on GitHub"}
                        </a>
                    </p>
                </article>
            </main>
        </>
    }
}
