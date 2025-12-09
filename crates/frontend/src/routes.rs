use crate::layouts::ShareHandler;
use crate::pages::{AddArticle, ArticleDetail, Home, Settings};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[not_found]
    #[at("/")]
    Home,
    #[at("/article/:id")]
    Article { id: i32 },
    #[at("/settings")]
    Settings,
    #[at("/add_article/")]
    AddArticle,
}

pub fn switch(routes: Route) -> Html {
    html! {
        <ShareHandler>
        {
            match routes {
                Route::Home => html! { <Home /> },
                Route::Article { id } => html! { <ArticleDetail {id} /> },
                Route::Settings => html! { <Settings /> },
                Route::AddArticle => html! { <AddArticle /> },
            }
        }
        </ShareHandler>
    }
}
