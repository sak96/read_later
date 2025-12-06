use crate::web_utils::sleep;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Html,
}

#[allow(dead_code)]
pub enum AlertStatus {
    Success,
    Info,
    Error,
}

#[derive(Clone, PartialEq)]
pub struct AlertContext {
    pub alert: Callback<(String, AlertStatus)>,
}

#[function_component(AlertHandler)]
pub fn theme_provider(props: &Props) -> Html {
    let message = use_state(Option::<String>::default);
    let status = use_state(|| AlertStatus::Info);
    let alert = {
        let message = message.clone();
        let status = status.clone();
        Callback::from(move |(new_message, new_status): (String, AlertStatus)| {
            let message = message.clone();
            let status = status.clone();
            status.set(new_status);
            message.set(Some(new_message));
            wasm_bindgen_futures::spawn_local(async move {
                sleep(5000).await;
                message.set(None);
            })
        })
    };

    let ctx = AlertContext { alert };
    html! {
    <>
        <ContextProvider<AlertContext> context={ctx}>
            { props.children.clone() }
            </ContextProvider<AlertContext>>
            {
                if let Some(text) = (*message).to_owned() {
                    let (bgcolor, color) = match *status {
                        AlertStatus::Info=>("#B7D9FC", "#017FC0"),
                        AlertStatus::Error=>("#F6CABF", "#D93526"),
                        AlertStatus::Success => ("#39F1A6", "#00895A"),
                    };
                    let style = format!("position: fixed; bottom: var(--safe-area-inset-bottom, 0); z-index: 1000; background-color: {bgcolor}; color: {color}");
                    html! {
                        <article class="pico container-fluid" {style}>{text}</article>
                    }
                } else {
                    html!{}
                }
            }
        </>
    }
}
