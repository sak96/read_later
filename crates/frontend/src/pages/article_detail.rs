use crate::components::{HomeButton, LinkPopup, SpeakBar};
use crate::layouts::{AlertContext, AlertStatus};
use crate::routes::Route;
use crate::web_utils::{invoke_no_parse_log_error, invoke_parse, open_url, set_callback_to_link};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ReadViewerProps {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub body: String,
    pub url: String,
}

#[function_component(ReadViewer)]
pub fn read_viewer(props: &ReadViewerProps) -> Html {
    // States
    let loading = use_state(|| true);
    let article_id = use_state(|| props.id);
    let title = use_state(String::new);
    let html_content = use_state(String::new);
    let url = use_state(String::new);
    let navigator = use_navigator().unwrap();
    let div_ref = use_node_ref();
    let delete_modal = use_state(|| false);
    let external_url = use_state(|| None::<String>);

    let on_link = {
        let external_url = external_url.clone();
        Callback::from(move |href: String| {
            external_url.set(Some(href));
        })
    };
    let on_link_close = {
        let external_url = external_url.clone();
        Callback::from(move |_| {
            external_url.set(None);
        })
    };

    // handle external link
    {
        let div_ref = div_ref.clone();
        let on_click = on_link.clone();
        use_effect_with(
            (div_ref, html_content.clone(), url.clone()),
            move |(div_ref, _, url)| {
                set_callback_to_link(div_ref, on_click, (*url).to_string());
            },
        )
    }

    // Load article on mount
    {
        let title = title.clone();
        let html_content = html_content.clone();
        let article_id = article_id.clone();
        let loading = loading.clone();
        let url = url.clone();
        let navigator = navigator.clone();
        let alert_ctx = use_context::<AlertContext>().expect("AlertContext missing");
        use_effect_with(article_id, move |article_id| {
            spawn_local({
                let article_id = article_id.clone();
                let navigator = navigator.clone();
                async move {
                    match invoke_parse::<Article>(
                        "get_article",
                        &Some(serde_json::json!({"id": *article_id})),
                    )
                    .await
                    {
                        Ok(article) => {
                            title.set(article.title);
                            html_content.set(article.body);
                            url.set(article.url);
                            loading.set(false);
                        }
                        Err(err) => {
                            alert_ctx.alert.emit((
                                format!("Failed to fetc article: {err}"),
                                AlertStatus::Error,
                            ));
                            navigator.push(&Route::Home);
                        }
                    }
                }
            });
            || ()
        });
    }

    let open_web_url = Callback::from(move |_| {
        let url = url.clone();
        spawn_local(async move {
            open_url((*url).to_owned()).await;
        });
    });

    let delete_article = {
        Callback::from(move |_| {
            let navigator = navigator.clone();
            let article_id = article_id.clone();
            spawn_local(async move {
                invoke_no_parse_log_error(
                    "delete_article",
                    &Some(serde_json::json!({"id": *article_id})),
                )
                .await;
                navigator.push(&Route::Home);
            });
        })
    };

    let delete_dialog_toggle = {
        let delete_modal = delete_modal.clone();
        Callback::from(move |_| {
            delete_modal.set(!*delete_modal);
        })
    };

    html! {
        <div class="container">
            <article ref={div_ref.clone()} style="min-height: 100vh" aria-busy={(*loading).to_string()}>
                <h1>{&*title}</h1>
                {Html::from_html_unchecked(((*html_content).clone()).into())}
            </article>
            // External Link handling
            <LinkPopup url={(*external_url).clone()} on_close={on_link_close} />
            // Delete modal
            <dialog open={*delete_modal}>
              <article>
                <h2><strong class="ti ti-trash-x">{format!("\u{f784}: {}", &*title)}</strong></h2>
                <footer>
                  <button class="secondary" onclick={delete_dialog_toggle.clone()}><i class="ti ti-x">{"\u{eb55}"}</i></button>
                  <button onclick={delete_article}><i class="ti ti-check">{"\u{ea5e}"}</i></button>
                </footer>
              </article>
            </dialog>
            // Action area
            <aside style="position: sticky; bottom: var(--safe-area-inset-bottom);">
                <nav>
                    <SpeakBar {div_ref} />
                    <div role="group">
                        <HomeButton />
                        <button onclick={open_web_url}><i class="ti ti-world-www">{"\u{f38f}"}</i></button>
                        <button class="secondary" onclick={delete_dialog_toggle}><i class="ti ti-trash-x">{"\u{f784}"}</i></button>
                    </div>
                </nav>
            </aside>
        </div>
    }
}
