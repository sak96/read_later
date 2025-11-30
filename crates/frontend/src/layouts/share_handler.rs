use crate::routes::Route;
use crate::web_utils::{ShareEvent, add_share_listener, is_android, remove_share_listener};
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShareHandlerProps {
    pub children: Html,
}

#[function_component(ShareHandler)]
pub fn share_handler(props: &ShareHandlerProps) -> Html {
    {
        let handler = use_state(Option::<u32>::default);
        let navigator = use_navigator().unwrap();
        let handler = handler.clone();
        use_effect(move || {
            if is_android() {
                let callback = {
                    Callback::from(move |e: ShareEvent| {
                        for part in e.uri.split(';') {
                            if let Some(eq_pos) = part.find('=') {
                                let (key, val) = part.split_at(eq_pos + 1);
                                if key.starts_with("S.android.intent.extra.TEXT") {
                                    let url = urlencoding::decode(val)
                                        .unwrap_or_else(|_| val.into())
                                        .into_owned();
                                    let mut query = HashMap::new();
                                    query.insert("input", url.to_string());
                                    navigator
                                        .push_with_query(&Route::AddArticle, &query)
                                        .unwrap();
                                }
                            }
                        }
                    })
                };
                {
                    let handler = handler.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let callback = callback.clone();
                        let id = add_share_listener(callback).await;
                        handler.set(Some(id));
                    });
                }
            }
            move || {
                if let Some(id) = handler.as_ref() {
                    let id = *id;
                    wasm_bindgen_futures::spawn_local(async move {
                        remove_share_listener(id).await;
                    });
                }
            }
        });
    }
    html! { <>{props.children.clone()}</> }
}
