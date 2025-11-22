use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[function_component(HomeButton)]
pub fn home_button() -> Html {
    let navigator = use_navigator().unwrap();
    let go_back = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });
    html! {
        <button onclick={go_back} class="secondary" style="position: sticky; top: 2em;">
            <i class="ti ti-arrow-left"></i>
        </button>
    }
}
