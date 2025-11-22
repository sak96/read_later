use crate::components::{AddArticleModal, ArticleCard, Fab};
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

    let toggle_fab_menu = {
        let show_fab_menu = show_fab_menu.clone();
        Callback::from(move |_| show_fab_menu.set(!*show_fab_menu))
    };

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

    let navigate_settings = use_navigator().unwrap();
    let go_to_settings = Callback::from(move |_| {
        navigate_settings.push(&Route::Settings);
    });

    html! {
        <>
            <main class="container">
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

            <Fab
                show_menu={*show_fab_menu}
                on_toggle={toggle_fab_menu}
                on_add={open_add_modal}
                on_settings={go_to_settings}
            />

           <AddArticleModal open={*show_modal} on_close={close_modal} />
        </>
    }
}
