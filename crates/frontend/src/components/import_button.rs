use crate::web_utils::import_data;
use yew::prelude::*;

#[component(ImportButton)]
pub fn import_button() -> Html {
    let open = use_state(|| false);
    let on_click = {
        let open = open.clone();
        Callback::from(move |_| {
            let open = open.clone();
            wasm_bindgen_futures::spawn_local(async move {
                open.set(true);
                import_data().await;
                open.set(false);
            })
        })
    };
    html! {
        <button onclick={on_click} type="button">
            <i class="ti ti-upload">{"\u{eb47}"}</i>
        </button>
    }
}
