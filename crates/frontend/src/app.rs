use crate::layouts::{AlertHandler, ThemeProvider};
use crate::routes::{Route, switch};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
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
