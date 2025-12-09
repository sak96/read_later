use crate::web_utils::export_data;
use yew::prelude::*;

#[component(ExportButton)]
pub fn export_button() -> Html {
    let open = use_state(|| false);
    let on_click = {
        let open = open.clone();
        Callback::from(move |_| {
            let open = open.clone();
            wasm_bindgen_futures::spawn_local(async move {
                open.set(true);
                export_data().await;
                open.set(false);
            })
        })
    };
    html! {
        <button onclick={on_click} type="button">
            <i class="sui sui-download">{"\u{f019}"}</i>
        </button>
    }
}
