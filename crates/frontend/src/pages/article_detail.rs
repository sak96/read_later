use crate::components::ReadViewer;
use crate::layouts::{AlertContext, AlertStatus};
use crate::routes::Route;
use crate::web_utils::{Channel, invoke_parse};
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

#[function_component(ArticleDetail)]
pub fn article_detail(props: &ArticleDetailProps) -> Html {
    let mode = use_state(|| PageMode::FetchingArticle(None));
    let navigator = use_navigator().unwrap();
    let progress_listener = use_mut_ref(|| None::<Channel<FetchProgress>>);

    // Load article on mount
    {
        let alert_ctx = use_context::<AlertContext>().expect("AlertContext missing");
        let mode = mode.clone();
        let navigator = navigator.clone();
        let progress_listener = progress_listener.clone();
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
                            alert_ctx.alert.emit((
                                format!("Failed to fetch article: {err}"),
                                AlertStatus::Error,
                            ));
                            navigator.push(&Route::Home);
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

    match &*mode {
        PageMode::FetchingArticle(event) => {
            html! {
                <main class="container page" style="display: flex; justify-content: center; align-items: center;">
                  <article style="width: 100%;">
                    {{
                       let (_icon, icon_code) = match event {
                        Some(FetchProgress::Downloading) => ("ti-cloud-download", "\u{ea71}"),
                        Some(FetchProgress::Parsing) => ("ti-database-search", "\u{fa18}"),
                        None => ("ti-loader", "\u{eca3}"),
                    };
                       html! {
                           <h2 class="ti">{icon_code}</h2>
                       }
                    }}
                    <progress />
                  </article>
                </main>
            }
        }
        PageMode::PageReturned(article) => html! {
            <ReadViewer article={article.clone()} />
        },
    }
}
