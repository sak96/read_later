use crate::routes::Route;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(
        js_namespace = ["window", "__TAURI__", "opener"],
        js_name = openUrl
    )]
    async fn open_url(url: JsValue) -> JsValue;
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
        Callback::from(move |value: String| {
            let theme = theme.clone();
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "name": "theme",
                    "value": value
                }))
                .unwrap();
                theme.set(value);
                invoke("set_setting", args).await;
            });
        })
    };

    let open_external_url = Callback::from(move |url: String| {
        spawn_local(async move {
            open_url(url.into()).await;
        });
    });

    web_sys::console::log_1(&(*theme).to_string().into());
    html! {
        <>
            <nav class="container-fluid">
                <ul>
                    <li><button onclick={go_back} class="secondary">
                        <i class="ti ti-arrow-left"></i>
                    </button></li>
                </ul>
            </nav>

            <main class="container">
                <article>
                    <label>
                        <h2 class="ti ti-palette"></h2>
                        <div role="group">
                            {
                                for [("light", "ti-sun"), ("dark","ti-moon"), ("system","ti-device-desktop-cog")].iter().map(|(theme_option, theme_icon)| {
                                    let btn_class = if *theme == *theme_option { "primary" } else { "outline" };
                                    let theme_value = theme_option.to_string();
                                    html! {
                                        <button
                                            class={classes!(btn_class)}
                                            onclick={on_theme_change.reform(move |_| theme_value.clone())}
                                        >
                                            <i class={classes!("ti", theme_icon.to_owned())}></i>
                                        </button>
                                    }
                                })
                            }
                        </div>
                    </label>
                </article>
                <article>
                    <label>
                        <h2 class="ti ti-info-circle"></h2>
                        <div role="group">
                            {
                                for [("https://github.com","ti-brand-github"), ("https://github.com","ti-bug")].iter().map(|(url, url_icon)| {
                                    html! {
                                        <button
                                            type="button"
                                            class="outline"
                                            onclick={open_external_url.reform(move |_| url.to_string())}
                                        >
                                            <i class={classes!("ti", url_icon.to_owned())}></i>
                                        </button>
                                    }
                                })
                            }
                        </div>
                    </label>
                </article>

                <article>
                    <div style="display: flex; gap: 8px;">
                    </div>
                </article>
            </main>
        </>
    }
}
