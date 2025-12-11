use crate::components::{HomeButton, SettingsButton};
use crate::layouts::{AlertContext, AlertStatus, Fab};
use crate::routes::Route;
use crate::web_utils::{invoke_parse, read_clipboard};
use shared::models::Article;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[component(AddArticle)]
pub fn add_article() -> Html {
    let url_input = use_state(String::new);
    let location = use_location().expect("no location");
    let progress_bar = use_state(|| false);
    {
        let url_input = url_input.clone();
        use_effect_with(location, move |location| {
            let location = location.clone();
            spawn_local(async move {
                if let Some(text) = location.state::<String>() {
                    url_input.set((*text).to_string());
                }
            })
        })
    };

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
                if let Some(text) = read_clipboard().await {
                    url_input.set(text)
                }
            });
        })
    };

    let navigator = use_navigator().unwrap();
    let alert_ctx = use_context::<AlertContext>().expect("AlertContext missing");
    let on_submit = {
        let url_input = url_input.clone();
        let progress_bar = progress_bar.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let url = (*url_input).clone();
            let navigator = navigator.clone();
            let progress_bar = progress_bar.clone();
            let alert_ctx = alert_ctx.clone();

            spawn_local(async move {
                progress_bar.set(true);
                match invoke_parse::<Article>("add_article", &Some(serde_json::json!({"url": url})))
                    .await
                {
                    Ok(article) => navigator.replace(&Route::Article { id: article.id }),
                    Err(err) => {
                        alert_ctx
                            .alert
                            .emit((format!("Failed to add article: {err}"), AlertStatus::Error));
                        progress_bar.set(false)
                    }
                }
            });
        })
    };

    html! {
        <article>
            if *progress_bar  {
                <blockquote>
                    {(*url_input).clone()}
                </blockquote>
                <article aria-busy="true"/>
            } else {
                <form class="container page" onsubmit={on_submit}>
                    <input
                        type="url"
                        value={(*url_input).clone()}
                        oninput={on_url_change}
                        placeholder="https://example.com/article"
                        required=true
                    />
                    <div role="group">
                        <button class="outline" type="button" onclick={paste_from_clipboard} >
                            <i class="ti ti-clipboard">{"\u{100cc}"}</i>
                        </button>
                        <div role="group">
                            <button type="submit">
                                <i class="ti ti-device-floppy">{"\u{eb62}"}</i>
                            </button>
                        </div>
                    </div>
                </form>
            }
            <Fab>
                <HomeButton />
                <SettingsButton />
            </Fab>
        </article>
    }
}
