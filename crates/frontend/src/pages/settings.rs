use crate::components::HomeButton;
use crate::layouts::ThemeContext;
use crate::web_utils::{invoke_no_parse_log_error, open_url};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(Settings)]
pub fn settings() -> Html {
    let theme = use_state(|| "system".to_string());
    let theme_ctx = use_context::<ThemeContext>().expect("ThemeProvider missing");

    {
        let theme = theme.clone();
        let theme_ctx = theme_ctx.clone();
        use_effect_with(theme, move |theme| theme.set(theme_ctx.mode.clone()));
    }

    let on_theme_change = {
        let theme = theme.clone();
        Callback::from(move |value: String| {
            let theme = theme.clone();
            let theme_ctx = theme_ctx.clone();
            spawn_local(async move {
                invoke_no_parse_log_error(
                    "set_setting",
                    &Some(serde_json::json!({ "name": "theme", "value": value })),
                )
                .await;
                theme.set(value.clone());
                theme_ctx.set_mode.emit(value);
            });
        })
    };

    let open_external_url = Callback::from(move |url: String| {
        spawn_local(async move {
            open_url(url).await;
        });
    });

    html! {
        <main class="container">
            <article>
                <label>
                    <h2 class="ti ti-palette"></h2>
                    <div role="group">
                        {
                            for [("light", "ti-sun"), ("dark","ti-moon"), ("system","ti-device-desktop-cog")].iter().map(|(theme_option, theme_icon)| {
                                html! {
                                    <button
                                        class={if *theme == *theme_option { "primary" } else { "outline" }}
                                        onclick={on_theme_change.reform(move |_| theme_option.to_string().clone())}
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
                            for [("https://github.com/sak96/read_later","ti-brand-github"), ("https://github.com/sak96/read_later/issues","ti-bug")].iter().map(|(url, url_icon)| {
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
            <div role="group">
                <HomeButton />
            </div>
        </main>
    }
}
