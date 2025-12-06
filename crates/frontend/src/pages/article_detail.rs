use crate::components::ReadViewer;
use crate::layouts::{AlertContext, AlertStatus};
use crate::routes::Route;
use crate::web_utils::invoke_parse;
use shared::models::Article;
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
    FetchingArticle,
    DownloadingUrl,
    ParsingPage,
    PageReturned(Article),
}

#[function_component(ArticleDetail)]
pub fn article_detail(props: &ArticleDetailProps) -> Html {
    let mode = use_state(|| PageMode::FetchingArticle);
    let navigator = use_navigator().unwrap();

    // Load article on mount
    {
        let alert_ctx = use_context::<AlertContext>().expect("AlertContext missing");
        let mode = mode.clone();
        use_effect_with(props.id, move |article_id| {
            let article_id = *article_id;
            let mode = mode.clone();
            spawn_local({
                let navigator = navigator.clone();
                async move {
                    mode.set(PageMode::FetchingArticle);
                    match invoke_parse::<Article>(
                        "get_article",
                        &Some(serde_json::json!({"id": article_id})),
                    )
                    .await
                    {
                        Ok(article) => {
                            mode.set(PageMode::PageReturned(article));
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

    match &*mode {
        PageMode::FetchingArticle => html! {
            <main class="container page" style="display: flex; justify-content: center; align-items: center;">
              <article style="width: 100%;">
                <h2 class="ti ti-loader">{"\u{eca3}"}</h2>
                <progress />
              </article>
            </main>
        },
        PageMode::DownloadingUrl => html! {
            <article aria-busy="true"><i class="ti ti-download">{"\u{ea96}"}</i></article>
        },
        PageMode::ParsingPage => html! {
            <article aria-busy="true"><i class="ti ti-ea96">{"\u{eca3}"}</i></article>
        },
        PageMode::PageReturned(article) => html! {
            <ReadViewer article={article.clone()} />
        },
    }
}
