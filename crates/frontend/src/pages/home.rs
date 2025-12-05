use crate::components::{ArticleCard, ArticleEntry, SettingsButton};
use crate::layouts::Fab;
use crate::routes::Route;
use crate::web_utils::invoke_parse_log_error;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    let articles = use_mut_ref(Vec::<ArticleEntry>::new);
    let loading = use_state(|| false);
    let force_update = use_force_update();
    let scroll_ref = use_node_ref();

    let navigator = use_navigator().unwrap();
    let add_article = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::AddArticle);
        })
    };
    let fetch_article = {
        let articles = articles.clone();
        let loading = loading.clone();
        let force_update = force_update.clone();
        Callback::from(move |_: ()| {
            loading.set(true);
            let articles = articles.clone();
            let force_update = force_update.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                if let Some(data) = invoke_parse_log_error::<Vec<ArticleEntry>>(
                    "get_articles",
                    &Some(serde_json::json!({ "offset": articles.borrow().len() / 2})),
                )
                .await
                {
                    if data.is_empty() {
                        if articles.borrow().is_empty() {
                            navigator.push(&Route::AddArticle);
                        } else {
                            loading.set(false);
                            return;
                        }
                    }
                    articles.borrow_mut().extend(data);
                    force_update.force_update();
                }
                loading.set(false);
            });
        })
    };

    let onscroll = {
        let loading = loading.clone();
        let fetch_article = fetch_article.clone();
        let scroll_ref = scroll_ref.clone();

        Callback::from(move |e: Event| {
            e.prevent_default();
            if *loading {
                return;
            }

            let target = scroll_ref.cast::<web_sys::HtmlElement>().unwrap();
            let scroll_top = target.scroll_top() as f64;
            let scroll_height = target.scroll_height() as f64;
            let client_height = target.client_height() as f64;
            if scroll_top + client_height > scroll_height - 100.0 {
                fetch_article.emit(());
            }
        })
    };
    {
        let fetch_article = fetch_article.clone();
        use_effect_with((), move |_| {
            fetch_article.emit(());
            move || {}
        });
    }

    html! {
        <>
            <main class="container"  ref={scroll_ref} {onscroll} style=r#"
                overflow: scroll;
                scroll-behaviour: smooth;
                height:  calc(100vh - var(--safe-area-inset-top) -  var(--safe-area-inset-bottom))
            "#>
                <div class="container" >
                    { for articles.borrow().iter().map(|article| html! {
                        <ArticleCard article={article.clone()} />
                    })}
                </div>
            </main>
            if *loading {
                <article aria-busy={true.to_string()} />
            }

            <Fab>
                <button onclick={add_article}>
                    <i class="ti ti-bookmark-plus">{"\u{fa60}"}</i>
                </button>
                <div>
                    <SettingsButton />
                </div>
            </Fab>
        </>
    }
}
