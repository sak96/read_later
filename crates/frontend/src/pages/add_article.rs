use crate::components::{HomeButton, SettingsButton};
use crate::layouts::Fab;
use crate::routes::Route;
use crate::web_utils::{invoke, read_clipboard};
use wasm_bindgen_futures::spawn_local;
use yew_router::prelude::*;

use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Default)]
struct ShareParams {
    input: Option<String>,
}

#[function_component(AddArticle)]
pub fn add_article() -> Html {
    let url_input = use_state(String::new);
    let location = use_location().expect("no location");
    let progress_bar = use_state(|| false);
    {
        let url_input = url_input.clone();
        use_effect_with(location, move |location| {
            let location = location.clone();
            spawn_local(async move {
                let query: ShareParams = location.query::<ShareParams>().unwrap_or_default();
                url_input.set(query.input.unwrap_or_default());
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
    let on_submit = {
        let url_input = url_input.clone();
        let progress_bar = progress_bar.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let url = (*url_input).clone();
            let navigator = navigator.clone();
            let progress_bar = progress_bar.clone();

            spawn_local(async move {
                progress_bar.set(true);
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({"url": url})).unwrap();
                invoke("add_article", args).await;
                navigator.push(&Route::Home);
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
                <form onsubmit={on_submit}>
                    <input
                        type="url"
                        value={(*url_input).clone()}
                        oninput={on_url_change}
                        placeholder="https://example.com/article"
                        required=true
                    />
                    <div role="group">
                        <button class="outline" type="button" onclick={paste_from_clipboard} >
                            <i class="ti ti-clipboard"></i>
                        </button>
                        <div role="group">
                            <button type="submit">
                                <i class="ti ti-check"></i>
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
