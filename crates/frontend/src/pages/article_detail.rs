use crate::components::ReadViewer;
use crate::layouts::{AlertContext, AlertStatus};
use crate::routes::Route;
use crate::web_utils::{Channel, invoke_no_parse_log_error, invoke_parse};
use shared::models::Article;
use shared::models::FetchProgress;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ArticleDetailProps {
    pub id: i32,
}

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum PageMode {
    FetchingArticle(Option<FetchProgress>),
    PageReturned(Article),
}

impl PageMode {
    pub fn is_done(&self) -> bool {
        matches!(&self, PageMode::PageReturned(_))
    }
}

#[component(ArticleDetail)]
pub fn article_detail(props: &ArticleDetailProps) -> Html {
    let mode = use_state(|| PageMode::FetchingArticle(None));
    let navigator = use_navigator().unwrap();
    let progress_listener = use_mut_ref(|| None::<Channel<FetchProgress>>);
    let alert_ctx = use_context::<AlertContext>().expect("AlertContext missing");

    // Load article on mount
    {
        let mode = mode.clone();
        let navigator = navigator.clone();
        let progress_listener = progress_listener.clone();
        let alert_ctx = alert_ctx.clone();
        use_effect_with(props.id, move |article_id| {
            let article_id = *article_id;
            let mode = mode.clone();
            let progress_listener = progress_listener.clone();
            spawn_local({
                let navigator = navigator.clone();

                let listener = {
                    let mode = mode.clone();
                    Callback::from(move |event: FetchProgress| {
                        if !mode.is_done() {
                            mode.set(PageMode::FetchingArticle(Some(event)));
                        }
                    })
                };
                let on_progress = Some(Channel::from(listener));
                progress_listener.replace(on_progress.clone());
                {
                    let progress_listener = progress_listener.clone();
                async move {
                    mode.set(PageMode::FetchingArticle(None));
                    let result = invoke_parse::<Article>(
                        "get_article",
                        &Some(serde_json::json!({"id": article_id, "onProgress": on_progress})),
                    )
                    .await;
                    match result {
                        Ok(article) => {
                            mode.set(PageMode::PageReturned(article));
                        }
                        Err(err) => {
                            if (progress_listener).borrow().is_some() {
                                alert_ctx.alert.emit((
                                    format!("Failed to fetch article: {err}"),
                                    AlertStatus::Error,
                                ));
                            }
                            spawn_local(async move {
                                invoke_no_parse_log_error(
                                    "delete_article",
                                    &Some(serde_json::json!({"id": article_id})),
                                )
                                .await;
                                navigator.push(&Route::Home);
                            });
                        }
                    }
                }
                }
            });
            let progress_listener = progress_listener.clone();
            move || {
                (*progress_listener).take();
            }
        });
    }

    // Handle delete
    let delete_article = {
        let article_id = props.id;
        let alert_ctx = alert_ctx.clone();
        Callback::from(move |_| {
            let navigator = navigator.clone();
            let alert_ctx = alert_ctx.clone();
            spawn_local(async move {
                invoke_no_parse_log_error(
                    "delete_article",
                    &Some(serde_json::json!({"id": article_id})),
                )
                .await;
                alert_ctx
                    .alert
                    .emit(("Deleted article.".to_string(), AlertStatus::Success));
                navigator.push(&Route::Home);
            });
        })
    };

    match &*mode {
        PageMode::FetchingArticle(event) => {
            html! {
                <main class="container page" style="display: flex; justify-content: center; align-items: center;">
                  <article style="width: 100%;">
                    {{
                       let (_icon, icon_code, title) = match event {
                        Some(FetchProgress::Downloading(title)) => ("sui-cloud sui-search", "\u{f0c2} \u{f002}:", title.to_string()),
                        Some(FetchProgress::Parsing(title)) => ("sui-code sui-print", "\u{f15b} \u{f02f}:", title.to_string()),
                        None => ("sui-database sui-search", "\u{f1c0} \u{f002}:", "...".to_string()),
                    };
                       html! {
                           <h2 class="sui">{icon_code}<p>{title}</p></h2>
                       }
                    }}
                    <progress />
                    if matches!(event, Some(FetchProgress::Downloading(_))) {
                            <footer dir="rtl">
                                <button class="secondary" onclick={delete_article}>
                                    <i class="sui sui-trash">{"\u{f1f8}"}</i>
                                </button>
                            </footer>
                    }
                  </article>
                </main>
            }
        }
        PageMode::PageReturned(article) => html! {
            <ReadViewer article={article.clone()} />
        },
    }
}
