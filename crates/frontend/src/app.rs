use crate::layouts::{AlertHandler, ThemeProvider};
use crate::routes::{Route, switch};
use crate::web_utils::set_inset;
use yew::prelude::*;
use yew_router::prelude::*;

#[component(App)]
pub fn app() -> Html {
    use_effect_with((), |_| {
        wasm_bindgen_futures::spawn_local(async move {
            set_inset().await;
        })
    });
    html! {
        <AlertHandler>
            <ThemeProvider>
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </ThemeProvider>
        </AlertHandler>
    }
}
