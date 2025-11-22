use crate::components::HomeButton;
use crate::pages::Article;
use crate::routes::Route;
use crate::web_utils::invoke;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: i32,
}

#[function_component(ArticleDetail)]
pub fn article_detail(props: &Props) -> Html {
    let article = use_state(|| None::<Article>);
    let rendered_content = use_state(String::new);

    let article_clone = article.clone();
    let id = props.id;

    {
        let rendered_content = rendered_content.clone();
        use_effect_with(id, move |id| {
            let id = *id;
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({"id": id})).unwrap();
                let result = invoke("get_article", args).await;
                if let Ok(data) = serde_wasm_bindgen::from_value::<Article>(result) {
                    rendered_content.set(data.body.clone());
                    article_clone.set(Some(data));
                }
            });
        });
    }

    let delete_article = {
        let navigator = use_navigator().unwrap();
        Callback::from(move |_| {
            let navigator = navigator.clone();
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({"id": id})).unwrap();
                invoke("delete_article", args).await;
                navigator.push(&Route::Home);
            });
        })
    };
    html! {
        <main class="container">
            <HomeButton />
            if let Some(article) = article.as_ref() {
                <article>
                    <h1>{&article.title}</h1>
                    <p><small>{&article.created_at}</small></p>
                    <hr />
                    <button type="button" onclick={delete_article} class="secondary">
                        <i class="ti ti-trash"></i>
                    </button>
                    {Html::from_html_unchecked(((*rendered_content).clone()).into())}
                </article>
            } else {
                <article aria-busy="true">{"Loading..."}</article>
            }
        </main>
    }
}
