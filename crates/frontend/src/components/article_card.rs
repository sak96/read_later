use crate::pages::Article;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ArticleCardProps {
    pub article: Article,
}

#[function_component(ArticleCard)]
pub fn article_card(props: &ArticleCardProps) -> Html {
    let navigator = use_navigator().unwrap();
    let article_id = props.article.id;

    let on_click = Callback::from(move |_| {
        navigator.push(&Route::Article { id: article_id });
    });

    html! {
        <article style="cursor: pointer;" onclick={on_click}>
            <h3>{&props.article.title}</h3>
            <p><small>{&props.article.created_at}</small></p>
        </article>
    }
}
