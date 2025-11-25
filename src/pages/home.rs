use crate::components::{ArticleCard, SettingsButton};
use crate::layouts::Fab;
use crate::routes::Route;
use crate::web_utils::invoke;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created_at: String,
}

#[function_component(Home)]
pub fn home() -> Html {
    let articles = use_state(Vec::<Article>::new);
    let refreshed = use_state(|| false);

    let navigator = use_navigator().unwrap();
    let add_article = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::AddArticle);
        })
    };

    let articles_clone = articles.clone();
    if !*refreshed {
        let refreshed = refreshed.clone();
        spawn_local(async move {
            let result = invoke("get_articles", JsValue::NULL).await;
            if let Ok(data) = serde_wasm_bindgen::from_value::<Vec<Article>>(result) {
                if data.is_empty() {
                    navigator.push(&Route::AddArticle);
                }
                articles_clone.set(data);
                refreshed.set(true);
            }
        });
    }

    html! {
        <>
            <main class="container">
                <div class="container">
                    { for articles.iter().map(|article| html! {
                        <ArticleCard article={article.clone()} />
                    })}
                </div>
            </main>

            <Fab>
                <button onclick={add_article}>
                    <i class="ti ti-plus"></i>
                </button>
                <div>
                    <SettingsButton />
                </div>
            </Fab>
        </>
    }
}
