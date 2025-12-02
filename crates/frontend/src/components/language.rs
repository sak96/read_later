use crate::web_utils::{invoke_no_parse, invoke_parse_log_error, stop_speak};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::Element;
use yew::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TTSVoice {
    pub id: String,
    pub name: String,
    pub lang: String,
    #[serde(default)]
    pub disabled: bool,
}

impl TTSVoice {
    fn label(&self) -> String {
        format!("{}_{}", self.name, self.lang)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVoicesResponse {
    pub voices: Vec<TTSVoice>,
}

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
                if let Some(voices) =
                    invoke_parse_log_error::<GetVoicesResponse>("plugin:tts|get_all_voices", &None)
                        .await
                {
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
                    if invoke_no_parse(
                        "plugin:tts|set_voice",
                        &Some(serde_json::json!({"voice": id})),
                    )
                    .await
                    .is_ok()
                    {
                        language.set(Some(num));
                    }
                });
            }
        })
    };
    let voice = format!(": {}", (*voice).as_ref().map_or("", |v| v.as_str()));
    html! {
        <>
            if languages.is_empty() && false {
                <> </>
            } else{
                <feildset>
                    <details ref={details_ref.clone()} class="dropdown" >
                        <summary role="button"><i class="ti ti-language"></i>{voice}</summary>
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
                </feildset>
            }
        </>
    }
}
