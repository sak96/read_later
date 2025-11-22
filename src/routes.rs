use crate::pages::{ArticleDetail, Home, Settings};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/article/:id")]
    Article { id: i32 },
    #[at("/settings")]
    Settings,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Article { id } => html! { <ArticleDetail {id} /> },
        Route::Settings => html! { <Settings /> },
    }
}

