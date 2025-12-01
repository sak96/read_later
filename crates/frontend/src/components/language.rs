use crate::web_utils::{invoke_no_parse, invoke_parse_log_error, stop_speak};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
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
        Callback::from(move |e: Event| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
            {
                let language = language.clone();
                if let Ok(num) = input.value().as_str().parse::<usize>()
                    && let Some(voice) = (*languages).get(num)
                {
                    let id = voice.id.clone();
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
            }
        })
    };
    html! {
        <>
            if languages.is_empty() {
                <> </>
            } else{
                <label role="button"><i class="ti ti-language"></i></label>
                <div role="group">
                    <select onchange={on_language_change} role="button" >
                        <option selected={language.is_none()} disabled={true} >{"---"}</option>
                        {languages.iter().enumerate().map(|(idx, lang)| {
                            html! {
                                <option value={idx.to_string()} selected={*language == Some(idx)}>
                                    {lang.label()}
                                </option>
                            }
                        }).collect::<Html>()}
                    </select>
                </div>
            }
        </>
    }
}
