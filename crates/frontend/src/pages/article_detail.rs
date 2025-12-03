use crate::components::{HomeButton, LanguageSelection, LinkPopup, SpeakRate};
use crate::layouts::{AlertContext, AlertStatus};
use crate::pages::Article;
use crate::routes::Route;
use crate::web_utils::{
    extract_text, find_visible_para_id, invoke_no_parse_log_error, invoke_parse, is_android,
    open_url, scroll_to_center, scroll_to_top, set_callback_to_link, speak, stop_speak,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewMode {
    View,
    Reader,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ReadViewerProps {
    pub id: i32,
}

#[function_component(ReadViewer)]
pub fn read_viewer(props: &ReadViewerProps) -> Html {
    // States
    let loading = use_state(|| true);
    let article_id = use_state(|| props.id);
    let title = use_state(String::new);
    let html_content = use_state(String::new);
    let url = use_state(String::new);
    let mode = use_state(|| ViewMode::View);
    let checkpoint = use_state(|| 0);
    let rate = use_state(|| 1.0);
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
    let on_rate_change = {
        let rate = rate.clone();
        Callback::from(move |new_rate| rate.set(new_rate))
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

    // Handle mode transitions
    let on_mode_switch = {
        let mode = mode.clone();
        let div_ref = div_ref.clone();
        let checkpoint = checkpoint.clone();
        Callback::from(move |_| {
            if *mode == ViewMode::Reader {
                spawn_local(stop_speak());
                scroll_to_top(&div_ref, *checkpoint);
                mode.set(ViewMode::View);
            } else {
                let id = find_visible_para_id();
                checkpoint.set(id);
                mode.set(ViewMode::Reader);
            }
        })
    };

    // Reader background task
    {
        let mode = mode.clone();
        let checkpoint = checkpoint.clone();
        let rate = rate.clone();
        let div_ref = div_ref.clone();
        use_effect_with((*mode, checkpoint), move |(reader_mode, checkpoint)| {
            if *reader_mode == ViewMode::Reader {
                let mode = mode.clone();
                let checkpoint = checkpoint.clone();
                let rate = rate.clone();
                let div_ref = div_ref.clone();
                spawn_local(async move {
                    if *mode == ViewMode::Reader {
                        if let Some(para_text) = extract_text(&div_ref, *checkpoint) {
                            let div_ref = div_ref.clone();
                            scroll_to_center(&div_ref, *checkpoint);
                            speak(para_text.clone(), *rate).await;
                            checkpoint.set(*checkpoint + 1);
                        } else {
                            mode.set(ViewMode::View);
                        }
                    }
                });
            }
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

    let scroll_to_checkpoint = {
        let div_ref = div_ref.clone();
        let checkpoint = checkpoint.clone();
        Callback::from(move |_| {
            scroll_to_top(&div_ref, *checkpoint);
        })
    };

    html! {
        <div class="container">
            <article ref={div_ref} style="min-height: 100vh" aria-busy={(*loading).to_string()}>
                <h1>{&*title}</h1>
                {Html::from_html_unchecked(((*html_content).clone()).into())}
            </article>
            if is_android() {
                <style>{{
                    let current_para = *checkpoint;
                    format!("#para_{current_para} {{border: var(--pico-border-width) solid var(--pico-primary-hover);border-radius: var(--pico-border-radius)}}")
                }}</style>
            }
            // External Link handling
            <LinkPopup url={(*external_url).clone()} on_close={on_link_close} />
            // Delete modal
            <dialog open={*delete_modal}>
              <article>
                <h2><i class="ti ti-trash" /><strong>{format!(": {}", &*title)}</strong></h2>
                <footer>
                  <button class="secondary" onclick={delete_dialog_toggle.clone()}><i class="ti ti-x"></i></button>
                  <button onclick={delete_article}><i class="ti ti-check"></i></button>
                </footer>
              </article>
            </dialog>
            // Action area
            <aside style="position: sticky; bottom: 0;">
                <nav>
                    if *mode == ViewMode::View {
                        if is_android() {
                            <fieldset role="group">
                                <button onclick={on_mode_switch.clone()}>
                                    <i class="ti ti-volume"></i>
                                </button>
                                <LanguageSelection />
                                <button onclick={scroll_to_checkpoint}><i class="ti ti-player-skip-back"></i></button>
                            </fieldset>
                        }
                        <div role="group">
                            <HomeButton />
                            <button onclick={open_web_url}><i class="ti ti-world-www"></i></button>
                            <button class="secondary" onclick={delete_dialog_toggle}><i class="ti ti-trash"></i></button>
                        </div>
                    } else {
                        <div role="group">
                            <button class="icon-btn pause-btn" onclick={on_mode_switch}>
                                <i class="ti ti-player-pause"></i>
                            </button>
                            <SpeakRate {on_rate_change} />
                        </div>
                    }
                </nav>
            </aside>
        </div>
    }
}
