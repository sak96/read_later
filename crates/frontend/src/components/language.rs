use crate::web_utils::{TTSVoice, get_voices, set_voice, stop_speak};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, HtmlSelectElement};
use yew::prelude::*;

#[component(LanguageSelection)]
pub fn language_selection() -> Html {
    let language = use_state(Option::<usize>::default);
    let languages = use_state(Vec::<TTSVoice>::new);
    let details_ref = use_node_ref();
    {
        let languages = languages.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(voices) = get_voices().await {
                    languages.set(voices.voices.into_iter().collect());
                }
            });
            || {
                spawn_local(stop_speak());
            }
        });
    }

    let on_language_change = {
        let languages = languages.clone();
        let language = language.clone();
        let details_ref = details_ref.clone();
        Callback::from(move |event: Event| {
            let target: Option<web_sys::EventTarget> = event.target();
            if let Some(select) = target.and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                && let Ok(num) = select.value().parse::<usize>()
            {
                let language = language.clone();
                let details_ref = details_ref.clone();
                if let Some(details) = details_ref.cast::<Element>() {
                    details.remove_attribute("open").ok();
                };
                if let Some(voice) = (*languages).get(num) {
                    let id = voice.id.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if set_voice(&id).await {
                            language.set(Some(num));
                        }
                    });
                }
            }
        })
    };
    html! {
        <>
            if languages.is_empty() {
                <> </>
            } else{
                <select role="button" onchange={on_language_change} class="ti">
                    <option selected={language.is_none()} disabled={true}><i class="ti ti-language">{"\u{ebbe}"}</i></option>
                    {
                        languages.iter().enumerate().map(|(idx, lang)| {
                        html! {
                            <option value={idx.to_string()}>{lang.label()}</option>
                        }
                    }).collect::<Html>()}
                </select>
            }
        </>
    }
}
