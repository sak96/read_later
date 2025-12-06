use crate::routes::Route;
use shared::models::ArticleEntry;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ArticleCardProps {
    pub article: ArticleEntry,
}

#[function_component(ArticleCard)]
pub fn article_card(props: &ArticleCardProps) -> Html {
    let navigator = use_navigator().unwrap();
    let article_id = props.article.id;

    let (title, loaded) = if props.article.title.is_empty() {
        if let Ok(url) = web_sys::Url::new(&props.article.url)
            && let Some(title) = url
                .pathname()
                .split('/')
                .filter(|s| !s.is_empty())
                .next_back()
                .map(|s| s.to_string())
        {
            (title, false)
        } else {
            ("Untitled".to_string(), false)
        }
    } else {
        (props.article.title.to_string(), true)
    };

    let on_click = Callback::from(move |_| {
        navigator.push(&Route::Article { id: article_id });
    });

    html! {
        <article style="cursor: pointer;" onclick={on_click}>
            <h3>
                if !loaded {
                    <i class="ti ti-loader">{"\u{eca3}"}</i>
                }
                {&title}
            </h3>
            <p><small>{&props.article.created_at}</small></p>
        </article>
    }
}
