use crate::components::{ExportButton, HomeButton, ImportButton, SpeakRate};
use crate::layouts::{Fab, ThemeContext};
use crate::web_utils::{get_setting, get_version, is_android, open_url, set_setting};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

const THEMES: [(&str, &str, &str); 3] = [
    ("light", "ti-sun", "\u{f6a9}"),
    ("dark", "ti-moon", "\u{eaf8}"),
    ("system", "ti-device-desktop-cog", "\u{f862}"),
];

const INFOS: [(&str, &str, &str); 2] = [
    (
        "https://github.com/sak96/read_later",
        "ti-brand-github",
        "\u{ec1c}",
    ),
    (
        "https://github.com/sak96/read_later/issues",
        "ti-bug",
        "\u{ea48}",
    ),
];

#[component(Settings)]
pub fn settings() -> Html {
    let theme_ctx = use_context::<ThemeContext>().expect("ThemeProvider missing");
    let version = use_state(|| "N/A".to_string());
    {
        let version = version.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(v) = get_version().await {
                    version.set(v)
                }
            })
        })
    }

    let on_theme_change = {
        let theme_ctx = theme_ctx.clone();
        Callback::from(move |value: String| {
            let theme_ctx = theme_ctx.clone();
            spawn_local(async move {
                set_setting("theme", &value).await;
                theme_ctx.set_mode.emit(value);
            });
        })
    };

    let open_external_url = Callback::from(move |url: String| {
        spawn_local(async move {
            open_url(url).await;
        });
    });

    let tts_enabled = use_state(|| true);
    {
        let tts_enabled = tts_enabled.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(value) = get_setting("tts").await
                    && let Ok(value) = value.parse::<bool>()
                {
                    tts_enabled.set(value);
                } else {
                    set_setting("tts", &is_android().to_string()).await;
                }
            });
        });
    }
    let tts_toggled = {
        let tts_enabled = tts_enabled.clone();
        Callback::from(move |_| {
            let tts_enabled = tts_enabled.clone();
            spawn_local(async move {
                let new_state = !*tts_enabled;
                set_setting("tts", &new_state.to_string()).await;
                tts_enabled.set(new_state);
            })
        })
    };

    html! {
        <article class="container page">
            <form class="container">
                <fieldset>
                    <label>
                        <h2 class="ti ti-palette">{"\u{eb01}"}</h2>
                        <div role="group">
                            for (theme_option, _theme_icon, theme_code) in THEMES {
                                        <button
                                            class={if theme_ctx.mode.eq(theme_option) { "primary" } else { "outline" }}
                                            onclick={on_theme_change.reform(move |_| theme_option.to_string().clone())}
                                        >
                                            <i class="ti">{theme_code}</i>
                                        </button>
                                    }
                        </div>
                    </label>
                </fieldset>
                if is_android(){
                    <fieldset>
                        <div role="group">
                            <tr style="background-color: var(--pico-mark-background-color)">
                                <th><h2 class="ti ti-volume">{"\u{eb51}"}</h2></th>
                                <td><input name="terms" type="checkbox" role="switch" onclick={tts_toggled} checked={*tts_enabled} /></td>
                            </tr>
                            <SpeakRate on_rate_change={Callback::from(|_| {})} outline={true}/>
                        </div>
                    </fieldset>
                }
                <fieldset>
                    <label>
                        <h2 class="ti ti-database-exclamation">{"\u{fa13}"}</h2>
                        <small>{"(beta)"}</small>
                        <div role="group">
                            <ImportButton />
                            <ExportButton />
                        </div>
                    </label>
                </fieldset>
                <fieldset>
                    <label>
                        <h2 class="ti ti-info-circle">{"\u{eac5}"}</h2>
                        <div role="group">
                            for (url, _url_icon, url_code) in &INFOS {
                                <button
                                    type="button"
                                    class="outline"
                                    onclick={open_external_url.reform(move |_| url.to_string())}
                                >
                                    <i class="ti">{*url_code}</i>
                                </button>
                            }
                        </div>
                    </label>
                </fieldset>
                <table>
                    <tbody>
                        <tr>
                            <th><i class="ti ti-tag">{"Version \u{ff02}"}</i></th>
                            <td>{(*version).to_owned()}</td>
                        </tr>
                        <tr>
                            <th><i class="ti ti-file-text-shield">{"\u{100f2}"}</i></th>
                            <td>
                                <a
                                    class="outline"
                                    onclick={open_external_url.reform(move |_| "https://github.com/sak96/read_later/blob/master/PRIVACY_POLICY.md".to_string())}>
                                    {"Last Updated: December 7, 2025"}
                                </a>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </form>
            <Fab>
                <HomeButton />
            </Fab>
        </article>
    }
}
