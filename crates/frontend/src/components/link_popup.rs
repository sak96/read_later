use crate::web_utils::open_url;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LinkPopupProps {
    pub url: Option<String>,
    pub on_close: Callback<()>,
}

#[component(LinkPopup)]
pub fn link_popup(props: &LinkPopupProps) -> Html {
    let open_external_url = {
        let url = props.url.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            let url = url.clone();
            let on_close = on_close.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(url) = url {
                    open_url(url).await;
                }
                on_close.emit(());
            });
        })
    };
    let on_close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };
    html! {
        <dialog open={props.url.is_some()}>
            if let Some(url) = props.url.as_ref() {
                <article>
                    <header>
                       <button aria-label="Close" onclick={on_close} rel="prev"></button>
                       <h2 class="sui sui-globe">{"\u{f0ac}"}</h2>
                    </header>
                    <strong>{url.to_owned()}</strong>
                    <footer>
                        <button type="button" onclick={open_external_url}>
                            <i class="sui sui-check">{"\u{f00c}"}</i>
                        </button>
                    </footer>
                </article>
            }
        </dialog>
    }
}
