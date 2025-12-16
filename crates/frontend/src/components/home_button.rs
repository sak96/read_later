use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[derive(Properties, PartialEq)]
pub struct HomeButtonProps {
    #[prop_or_default]
    pub replace_history: bool,
}

#[component(HomeButton)]
pub fn home_button(props: &HomeButtonProps) -> Html {
    let navigator = use_navigator().unwrap();
    let replace_history = props.replace_history;
    let on_click = {
        Callback::from(move |_| {
            if replace_history {
                navigator.replace(&Route::Home);
            } else {
                navigator.push(&Route::Home);
            }
        })
    };
    html! {
        <button onclick={on_click}>
            <i class="ti ti-home">{"\u{eac1}"}</i>
        </button>
    }
}
