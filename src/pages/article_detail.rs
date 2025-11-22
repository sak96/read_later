use crate::pages::Article;
use crate::routes::Route;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: i32,
}

#[function_component(ArticleDetail)]
pub fn article_detail(props: &Props) -> Html {
    let article = use_state(|| None::<Article>);
    let rendered_content = use_state(|| String::new());

    let article_clone = article.clone();
    let rendered_clone = rendered_content.clone();
    let id = props.id;

    use_effect_with(id, move |id| {
        let id = id.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({"id": id})).unwrap();
            let result = invoke("get_article", args).await;

            if let Ok(data) = serde_wasm_bindgen::from_value::<Article>(result) {
                let body_args =
                    serde_wasm_bindgen::to_value(&serde_json::json!({"html": data.body})).unwrap();
                let rendered = invoke("render_readable_content", body_args).await;

                if let Ok(content) = serde_wasm_bindgen::from_value::<String>(rendered) {
                    rendered_clone.set(content);
                }
                article_clone.set(Some(data));
            }
        });
    });

    let navigator = use_navigator().unwrap();
    let go_back = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });

    html! {
        <>
            <nav class="container-fluid">
                <ul>
                    <li><button onclick={go_back} class="secondary">
                        <i class="ti ti-arrow-left"></i>
                    </button></li>
                    <li><strong>{"Article"}</strong></li>
                </ul>
            </nav>

            <main class="container">
                if let Some(article) = article.as_ref() {
                    <article>
                        <h1>{&article.title}</h1>
                        <p><small>{&article.created_at}</small></p>
                        <hr />
                        <div dangerouslySetInnerHTML={(*rendered_content).clone()} />
                    </article>
                } else {
                    <article aria-busy="true">{"Loading..."}</article>
                }
            </main>
        </>
    }
}
