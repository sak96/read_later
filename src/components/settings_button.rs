use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[function_component(SettingsButton)]
pub fn settings_button() -> Html {
    let navigator = use_navigator().unwrap();
    let on_click = Callback::from(move |_| {
        navigator.push(&Route::Settings);
    });
    html! {
        <button onclick={on_click} >
            <i class="ti ti-settings-filled"></i>
        </button>
    }
}
