use crate::web_utils::{TTSVoice, get_voices, set_voice, stop_speak};
use wasm_bindgen_futures::spawn_local;
use web_sys::Element;
use yew::prelude::*;

#[function_component(LanguageSelection)]
pub fn language_selection() -> Html {
    let language = use_state(Option::<usize>::default);
    let languages = use_state(Vec::<TTSVoice>::new);
    let details_ref = use_node_ref();
    let voice = use_state(|| None::<String>);
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
        let voice = voice.clone();
        let details_ref = details_ref.clone();
        Callback::from(move |num: usize| {
            let language = language.clone();
            let voice_ = voice.clone();
            let details_ref = details_ref.clone();
            if let Some(details) = details_ref.cast::<Element>() {
                details.remove_attribute("open").ok();
            };
            if let Some(voice) = (*languages).get(num) {
                let id = voice.id.clone();
                voice_.set(Some(voice.label()));
                wasm_bindgen_futures::spawn_local(async move {
                    if set_voice(&id).await {
                        language.set(Some(num));
                    }
                });
            }
        })
    };
    let voice = format!(": {}", (*voice).as_ref().map_or("", |v| v.as_str()));
    html! {
        <>
            if languages.is_empty() {
                <> </>
            } else{
                <div role="button">
                    <details ref={details_ref.clone()} class="dropdown" >
                        <summary role="button"><i class="ti ti-language">{"\u{ebbe}"}</i>{voice}</summary>
                        <ul>
                            {
                                languages.iter().enumerate().map(|(idx, lang)| {
                                html! {
                                    <li onclick={on_language_change.clone().reform(move |_| idx)}>
                                        {lang.label()}
                                    </li>
                                }
                            }).collect::<Html>()}
                        </ul>
                    </details>
                </div>
            }
        </>
    }
}
