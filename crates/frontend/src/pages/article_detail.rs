use crate::components::HomeButton;
use crate::layouts::{AlertContext, AlertStatus};
use crate::pages::Article;
use crate::routes::Route;
use crate::web_utils::{
    extract_text, find_visible_para_id, invoke_no_parse, invoke_no_parse_log_error, invoke_parse,
    invoke_parse_log_error, is_android, open_url, scroll_to_center, scroll_to_top, speak,
    stop_speak,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewMode {
    View,
    Reader,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpeechRate {
    Quarter, // 0.25
    Half,    // 0.5
    Normal,  // 1.0
    Double,  // 2.0
    Quad,    // 4.0
}

impl SpeechRate {
    fn as_f32(&self) -> f32 {
        match self {
            SpeechRate::Quarter => 0.25,
            SpeechRate::Half => 0.5,
            SpeechRate::Normal => 1.0,
            SpeechRate::Double => 2.0,
            SpeechRate::Quad => 4.0,
        }
    }

    fn all() -> Vec<Self> {
        vec![
            SpeechRate::Quarter,
            SpeechRate::Half,
            SpeechRate::Normal,
            SpeechRate::Double,
            SpeechRate::Quad,
        ]
    }

    fn label(&self) -> &str {
        match self {
            SpeechRate::Quarter => "0.25x",
            SpeechRate::Half => "0.50x",
            SpeechRate::Normal => "1.00x",
            SpeechRate::Double => "2.00x",
            SpeechRate::Quad => "4.00x",
        }
    }
}

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

#[derive(Properties, PartialEq, Clone)]
pub struct ReadViewerProps {
    pub id: i32,
}

#[function_component(ReadViewer)]
pub fn read_viewer(props: &ReadViewerProps) -> Html {
    // States
    let loading = use_state(|| true);
    let article_id = use_state(|| props.id);
    let title = use_state(String::new);
    let html_content = use_state(String::new);
    let url = use_state(String::new);
    let mode = use_state(|| ViewMode::View);
    let checkpoint = use_state(|| 0);
    let rate = use_state(|| SpeechRate::Normal);
    let language = use_state(Option::<usize>::default);
    let languages = use_state(Vec::<TTSVoice>::new);
    let navigator = use_navigator().unwrap();

    // load language on mount
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
            })
        });
    }

    // Load article on mount
    {
        let title = title.clone();
        let html_content = html_content.clone();
        let article_id = article_id.clone();
        let loading = loading.clone();
        let url = url.clone();
        let navigator = navigator.clone();
        let alert_ctx = use_context::<AlertContext>().expect("AlertContext missing");
        use_effect_with(article_id, move |article_id| {
            spawn_local({
                let article_id = article_id.clone();
                let navigator = navigator.clone();
                async move {
                    match invoke_parse::<Article>(
                        "get_article",
                        &Some(serde_json::json!({"id": *article_id})),
                    )
                    .await
                    {
                        Ok(article) => {
                            title.set(article.title);
                            html_content.set(article.body);
                            url.set(article.url);
                            loading.set(false);
                        }
                        Err(err) => {
                            alert_ctx.alert.emit((
                                format!("Failed to fetc article: {err}"),
                                AlertStatus::Error,
                            ));
                            navigator.push(&Route::Home);
                        }
                    }
                }
            });
            || ()
        });
    }

    // Handle mode transitions
    let on_mode_switch = {
        let mode = mode.clone();
        let checkpoint = checkpoint.clone();
        Callback::from(move |_| {
            if *mode == ViewMode::Reader {
                spawn_local(stop_speak());
                mode.set(ViewMode::View);
            } else {
                scroll_to_top(*checkpoint);
                let id = find_visible_para_id();
                checkpoint.set(id);
                mode.set(ViewMode::Reader);
            }
        })
    };

    // Reader background task
    {
        let mode = mode.clone();
        let checkpoint = checkpoint.clone();
        let rate = rate.clone();
        use_effect_with((*mode, checkpoint), move |(reader_mode, checkpoint)| {
            if *reader_mode == ViewMode::Reader {
                let mode = mode.clone();
                let checkpoint = checkpoint.clone();
                let rate = rate.clone();
                spawn_local(async move {
                    if *mode == ViewMode::Reader {
                        if let Some(para_text) = extract_text(*checkpoint) {
                            scroll_to_center(*checkpoint);
                            speak(para_text.clone(), rate.as_f32()).await;
                            checkpoint.set(*checkpoint + 1);
                        } else {
                            mode.set(ViewMode::View);
                        }
                    }
                });
            }
            || ()
        });
    }

    let open_web_url = Callback::from(move |_| {
        let url = url.clone();
        spawn_local(async move {
            open_url((*url).to_owned()).await;
        });
    });

    let delete_article = {
        Callback::from(move |_| {
            let navigator = navigator.clone();
            let article_id = article_id.clone();
            spawn_local(async move {
                invoke_no_parse_log_error(
                    "delete_article",
                    &Some(serde_json::json!({"id": *article_id})),
                )
                .await;
                navigator.push(&Route::Home);
            });
        })
    };

    let scroll_to_checkpoint = {
        let checkpoint = checkpoint.clone();
        Callback::from(move |_| {
            scroll_to_top(*checkpoint);
        })
    };

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
                    spawn_local(async move {
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
        <div class="container">
            if *loading {
                <article aria-busy="true"/>
            } else {
                <article >
                    <h1>{&*title}</h1>
                    {Html::from_html_unchecked(((*html_content).clone()).into())}
                </article>
            }
            if is_android() {
                <style>{{
                    let current_para = *checkpoint;
                    format!("#para_{current_para} {{border: var(--pico-border-width) solid var(--pico-primary-hover);border-radius: var(--pico-border-radius)}}")
                }}</style>
            }
            // Action area
            <aside style="position: sticky; bottom: 0;">
                <nav>
                    if *mode == ViewMode::View {
                        if is_android() {
                            <div role="group">
                                <button class="icon-btn" onclick={on_mode_switch.clone()}>
                                    <i class="ti ti-player-play"></i>
                                </button>
                                <select disabled={languages.is_empty()} onchange={on_language_change} role="button" >
                                    <option selected={language.is_none()} disabled={true} >{" 🔡"}</option>
                                    {languages.iter().enumerate().map(|(idx, lang)| {
                                        html! {
                                            <option value={idx.to_string()} selected={*language == Some(idx)}>
                                                {lang.label()}
                                            </option>
                                        }
                                    }).collect::<Html>()}
                                </select>
                                <button onclick={scroll_to_checkpoint}><i class="ti ti-restore"></i></button>
                            </div>
                        }
                        <div role="group">
                            <HomeButton />
                            <button onclick={open_web_url}><i class="ti ti-world-www"></i></button>
                            <button class="secondary" onclick={delete_article}><i class="ti ti-trash"></i></button>
                        </div>
                    } else {
                        <div role="group">
                            <button class="icon-btn pause-btn" onclick={on_mode_switch}>
                                <i class="ti ti-player-pause"></i>
                            </button>
                            <div role="group">
                                <label role="button">
                                    <b>{rate.label()}</b>
                                    <input
                                        type="range"
                                        min="0"
                                        max={SpeechRate::all().len().saturating_sub(1).to_string()}
                                        value={SpeechRate::all().iter().position(|r| r == &*rate).unwrap_or(2).to_string()}
                                        onchange={
                                            let rate = rate.clone();
                                            Callback::from(move |e: Event| {
                                                if let Some(input) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) && let Ok(idx) = input.value().parse::<usize>()  && let Some(new_rate) = SpeechRate::all().get(idx) {
                                                    rate.set(*new_rate);
                                                }
                                            })
                                        }
                                    />
                                </label>
                            </div>
                        </div>
                    }
                </nav>
            </aside>
        </div>
    }
}
