use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[derive(Properties, PartialEq)]
pub struct HomeButtonProps {
    #[prop_or_default]
    pub outline: bool,
}

#[function_component(HomeButton)]
pub fn home_button(props: &HomeButtonProps) -> Html {
    let navigator = use_navigator().unwrap();
    let on_click = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });
    let outline_class = props.outline.then_some("outline");
    html! {
        <button onclick={on_click} class={classes!(outline_class)}>
            <i class="ti ti-home"></i>
        </button>
    }
}
