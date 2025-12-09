use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[component(HomeButton)]
pub fn home_button() -> Html {
    let navigator = use_navigator().unwrap();
    let on_click = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });
    html! {
        <button onclick={on_click}>
            <i class="ti ti-home">{"\u{eac1}"}</i>
        </button>
    }
}
