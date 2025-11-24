use crate::components::{AddArticleModal, ArticleCard, SettingsButton};
use crate::layouts::Fab;
use crate::web_utils::invoke;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

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
    let show_modal = use_state(|| false);
    let show_fab_menu = use_state(|| false);
    let refreshed = use_state(|| false);

    let articles_clone = articles.clone();
    if !*refreshed {
        let refreshed = refreshed.clone();
        spawn_local(async move {
            let result = invoke("get_articles", JsValue::NULL).await;
            if let Ok(data) = serde_wasm_bindgen::from_value::<Vec<Article>>(result) {
                articles_clone.set(data);
                refreshed.set(true);
            }
        });
    }

    let open_add_modal = {
        let show_modal = show_modal.clone();
        let show_fab_menu = show_fab_menu.clone();
        Callback::from(move |_| {
            show_modal.set(true);
            show_fab_menu.set(false);
        })
    };

    let close_modal = {
        let show_modal = show_modal.clone();
        Callback::from(move |_| {
            refreshed.set(false);
            show_modal.set(false);
        })
    };

    html! {
        <>
            <main class="container"  style="min-height: 100vh">
                if articles.is_empty() {
                    <article >
                        <header>
                            <button type="submit" onclick={open_add_modal.clone()}>
                                <i class="ti ti-table-plus"></i>
                            </button>
                        </header>
                    </article>
                } else {
                    <div class="container">
                        { for articles.iter().map(|article| html! {
                            <ArticleCard article={article.clone()} />
                        })}
                    </div>
                }
            </main>

            <Fab>
                <div>
                    <button onclick={open_add_modal}>
                        <i class="ti ti-plus"></i>
                    </button>
                </div>
                <div>
                    <SettingsButton />
                </div>
            </Fab>

           <AddArticleModal open={*show_modal} on_close={close_modal} />
        </>
    }
}
