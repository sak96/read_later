use crate::components::HomeButton;
use crate::pages::Article;
use crate::routes::Route;
use crate::web_utils::{
    extract_text, find_visible_para_id, scroll_to_center, scroll_to_top, speak, stop_speak,
};
use crate::web_utils::{invoke, ostype};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    EnUS,
    EnGB,
}

impl Language {
    fn as_str(&self) -> &str {
        match self {
            Language::EnUS => "en_US",
            Language::EnGB => "en_GB",
        }
    }

    fn all() -> Vec<Self> {
        vec![Language::EnUS, Language::EnGB]
    }

    fn label(&self) -> &str {
        match self {
            Language::EnUS => "English (US)",
            Language::EnGB => "English (UK)",
        }
    }
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
    let mode = use_state(|| ViewMode::View);
    let checkpoint = use_state(|| 0);
    let rate = use_state(|| SpeechRate::Normal);
    let language = use_state(|| Language::EnUS);

    // Load article on mount
    {
        let title = title.clone();
        let html_content = html_content.clone();
        let article_id = article_id.clone();
        let loading = loading.clone();

        use_effect_with(article_id, move |article_id| {
            spawn_local({
                let title = title.clone();
                let html_content = html_content.clone();
                let article_id = article_id.clone();

                async move {
                    let args =
                        serde_wasm_bindgen::to_value(&serde_json::json!({"id": *article_id}))
                            .unwrap();
                    let result = invoke("get_article", args).await;
                    if let Ok(article) = serde_wasm_bindgen::from_value::<Article>(result) {
                        title.set(article.title);
                        html_content.set(article.body);
                        loading.set(false);
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
        let language = language.clone();

        use_effect_with((*mode, checkpoint), move |(reader_mode, checkpoint)| {
            if *reader_mode == ViewMode::Reader {
                let mode = mode.clone();
                let checkpoint = checkpoint.clone();
                let rate = rate.clone();
                let language = language.clone();
                spawn_local(async move {
                    if *mode == ViewMode::Reader {
                        if let Some(para_text) = extract_text(*checkpoint) {
                            scroll_to_center(*checkpoint);
                            speak(
                                para_text.clone(),
                                rate.as_f32(),
                                language.as_str().to_string(),
                            )
                            .await;
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

    let delete_article = {
        let navigator = use_navigator().unwrap();
        Callback::from(move |_| {
            let navigator = navigator.clone();
            let article_id = article_id.clone();
            spawn_local(async move {
                let args =
                    serde_wasm_bindgen::to_value(&serde_json::json!({"id": *article_id})).unwrap();
                invoke("delete_article", args).await;
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
        let language = language.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
            {
                match input.value().as_str() {
                    "en_US" => language.set(Language::EnUS),
                    "en_GB" => language.set(Language::EnGB),
                    _ => {}
                }
            }
        })
    };
    // <article aria-busy="true"></article>
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
                if ostype().eq(&"android") {
                    <style>{{
                        let current_para = *checkpoint;
                        format!("#para_{current_para} {{border: var(--pico-border-width) solid var(--pico-primary-hover);border-radius: var(--pico-border-radius)}}")
                    }}</style>
                }

                // Action area
                <aside style="position: sticky; bottom: 0;">
                <nav>
                    if *mode == ViewMode::View {
                        if ostype().eq(&"android") {
                            <div role="group">
                                <button class="icon-btn" onclick={on_mode_switch.clone()}>
                                    <i class="ti ti-player-play"></i>
                                </button>
                                <select onchange={on_language_change} role="button" >
                                    {Language::all().into_iter().map(|lang| {
                                        html! {
                                            <option
                                                value={lang.as_str().to_owned()}
                                                selected={*language == lang}
                                            >
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
