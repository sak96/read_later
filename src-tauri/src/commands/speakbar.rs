use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use tauri_plugin_tts::TtsExt;

#[cfg(any(target_os = "android", target_os = "ios"))]
use tauri_plugin_media_session::{MediaAction, MediaSessionExt, MediaState};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Mode {
    #[serde(rename = "view")]
    View,
    #[serde(rename = "reader")]
    Reader,
    Skipto(usize),
}

impl Mode {
    pub fn to_frontend(&self) -> Mode {
        match self {
            Mode::Skipto(_) => Mode::Reader,
            other => other.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateChanged {
    pub position: Option<usize>,
    pub mode: Mode,
}

pub struct SpeakBarState {
    pub paragraphs: RwLock<Vec<String>>,
    pub title: RwLock<String>,
    pub current_position: RwLock<usize>,
    pub rate: RwLock<f32>,
    pub voice_id: RwLock<Option<String>>,
    pub mode: RwLock<Mode>,
    pub tts_listener_ids: RwLock<Vec<u32>>,
    pub cumulative_durations: RwLock<Vec<f64>>,
    pub total_duration: RwLock<f64>,
}

impl Default for SpeakBarState {
    fn default() -> Self {
        Self {
            paragraphs: RwLock::new(Vec::new()),
            title: RwLock::new(String::new()),
            current_position: RwLock::new(0),
            rate: RwLock::new(1.0),
            voice_id: RwLock::new(None),
            mode: RwLock::new(Mode::View),
            tts_listener_ids: RwLock::new(Vec::new()),
            cumulative_durations: RwLock::new(Vec::new()),
            total_duration: RwLock::new(0.0),
        }
    }
}

fn compute_cumulative_durations(paragraphs: &[String]) -> (Vec<f64>, f64) {
    let mut cumulative = Vec::with_capacity(paragraphs.len());
    let mut running = 0.0_f64;
    for text in paragraphs {
        let word_count = text.split_whitespace().count() as f64;
        let duration = 4.0 * (word_count / 10.0);
        running += duration;
        cumulative.push(running);
    }
    (cumulative, running)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadState {
    pub mode: Mode,
    pub position: usize,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn setup_media_action_listener(app: &AppHandle) {
    let app_clone = app.clone();
    app.media_session().on_action(move |event| {
        let app = app_clone.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(state) = app.try_state::<SpeakBarState>() {
                match event.action {
                    MediaAction::Play => {
                        let mode = state
                            .mode
                            .read()
                            .map_err(|e| e.to_string())
                            .unwrap()
                            .clone();
                        if mode == Mode::View {
                            *state
                                .mode
                                .write()
                                .map_err(|e| e.to_string())
                                .unwrap() = Mode::Reader;
                            let _ = update_media_session(&app).await;
                            let pos = *state
                                .current_position
                                .read()
                                .map_err(|e| e.to_string())
                                .unwrap();
                            let _ = app.emit(
                                "speakbar:state-changed",
                                StateChanged {
                                    position: Some(pos),
                                    mode: Mode::Reader,
                                },
                            );
                        }
                    }
                    MediaAction::Pause | MediaAction::Stop => {
                        let mode = state
                            .mode
                            .read()
                            .map_err(|e| e.to_string())
                            .unwrap()
                            .clone();
                        if mode != Mode::View {
                            let app2 = app.clone();
                            let _ = stop_reading(app2, state).await;
                        }
                    }
                    MediaAction::Seek => {
                        if let Some(seek_pos) = event.seek_position {
                            handle_seek(&app, seek_pos).await;
                        }
                    }
                    _ => {}
                }
            }
        });
    });
}

#[cfg(any(target_os = "android", target_os = "ios"))]
async fn handle_seek(app: &AppHandle, seek_pos: f64) {
    if let Some(state) = app.try_state::<SpeakBarState>() {
        let rate = state.rate.read().map(|r| *r).unwrap_or(1.0);
        let durations = state
            .cumulative_durations
            .read()
            .map(|d| d.clone())
            .unwrap_or_default();

        let target = durations
            .iter()
            .position(|&d| d / rate as f64 >= seek_pos)
            .unwrap_or(durations.len().saturating_sub(1));

        let mode = state
            .mode
            .read()
            .map_err(|e| e.to_string())
            .unwrap()
            .clone();

        match mode {
            Mode::Reader | Mode::Skipto(_) => {
                *state
                    .mode
                    .write()
                    .map_err(|e| e.to_string())
                    .unwrap() = Mode::Skipto(target);
                let _ = update_media_session(app).await;
                let _ = app.emit(
                    "speakbar:state-changed",
                    StateChanged {
                        position: Some(target),
                        mode: Mode::Reader,
                    },
                );
                app.tts().stop();
            }
            Mode::View => {
                *state
                    .current_position
                    .write()
                    .map_err(|e| e.to_string())
                    .unwrap() = target;
                *state
                    .mode
                    .write()
                    .map_err(|e| e.to_string())
                    .unwrap() = Mode::Reader;
                let _ = update_media_session(app).await;
                let _ = app.emit(
                    "speakbar:state-changed",
                    StateChanged {
                        position: Some(target),
                        mode: Mode::Reader,
                    },
                );
            }
        }
    }
}

#[tauri::command]
pub async fn init_reading(
    app: AppHandle,
    rate: f32,
    title: String,
    paragraphs: Vec<String>,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    let (cumulative, total) = compute_cumulative_durations(&paragraphs);
    *state.paragraphs.write().map_err(|e| e.to_string())? = paragraphs;
    *state.title.write().map_err(|e| e.to_string())? = title;
    *state.rate.write().map_err(|e| e.to_string())? = rate;
    *state
        .current_position
        .write()
        .map_err(|e| e.to_string())? = 0;
    *state
        .cumulative_durations
        .write()
        .map_err(|e| e.to_string())? = cumulative;
    *state
        .total_duration
        .write()
        .map_err(|e| e.to_string())? = total;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = update_media_session(&app).await;
        setup_media_action_listener(&app);
    }

    let listener_finish = {
        let app_clone = app.clone();
        app_clone.clone().listen("tts://speech:finish", {
            move |_event: tauri::Event| {
                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app.try_state::<SpeakBarState>() {
                        let current_mode = state
                            .mode
                            .read()
                            .map_err(|e| e.to_string())
                            .unwrap()
                            .clone();
                        let pos = *state
                            .current_position
                            .read()
                            .map_err(|e| e.to_string())
                            .unwrap();

                        let (next_pos, reset_mode) = match &current_mode {
                            Mode::Skipto(target) => (*target, true),
                            _ => (pos + 1, false),
                        };

                        *state
                            .current_position
                            .write()
                            .map_err(|e| e.to_string())
                            .unwrap() = next_pos;
                        if reset_mode {
                            *state
                                .mode
                                .write()
                                .map_err(|e| e.to_string())
                                .unwrap() = Mode::Reader;
                        }

                        #[cfg(any(target_os = "android", target_os = "ios"))]
                        let _ = update_media_session(&app).await;

                        let app = app.clone();
                        let _ = start_reading(app, None, state).await;
                    }
                });
            }
        })
    };

    let listener_error = {
        let app_clone = app.clone();
        app_clone.clone().listen("tts://speech:error", {
            move |_event: tauri::Event| {
                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app.try_state::<SpeakBarState>() {
                        let app = app.clone();
                        let _ = stop_reading(app, state).await;
                    }
                });
            }
        })
    };

    let listener_interrupted = {
        let app_clone = app.clone();
        app_clone.clone().listen("tts://speech:interrupted", {
            move |_event: tauri::Event| {
                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app.try_state::<SpeakBarState>() {
                        let current_mode = state
                            .mode
                            .read()
                            .map_err(|e| e.to_string())
                            .unwrap()
                            .clone();
                        match current_mode {
                            Mode::Skipto(target) => {
                                *state
                                    .current_position
                                    .write()
                                    .map_err(|e| e.to_string())
                                    .unwrap() = target;
                                *state
                                    .mode
                                    .write()
                                    .map_err(|e| e.to_string())
                                    .unwrap() = Mode::Reader;
                                #[cfg(any(target_os = "android", target_os = "ios"))]
                                let _ = update_media_session(&app).await;
                                let _ = app.emit(
                                    "speakbar:state-changed",
                                    StateChanged {
                                        position: Some(target),
                                        mode: Mode::Reader,
                                    },
                                );
                                let app2 = app.clone();
                                let _ = start_reading(app2, None, state).await;
                            }
                            Mode::Reader => {
                                let _ = stop_reading(app, state).await;
                            }
                            Mode::View => {}
                        }
                    }
                });
            }
        })
    };

    state
        .tts_listener_ids
        .write()
        .map_err(|e| e.to_string())?
        .clear();
    state
        .tts_listener_ids
        .write()
        .map_err(|e| e.to_string())?
        .push(listener_finish);
    state
        .tts_listener_ids
        .write()
        .map_err(|e| e.to_string())?
        .push(listener_error);
    state
        .tts_listener_ids
        .write()
        .map_err(|e| e.to_string())?
        .push(listener_interrupted);

    Ok(())
}

#[tauri::command]
pub async fn start_reading(
    app: AppHandle,
    start_para: Option<usize>,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    let len = {
        let paragraphs = state.paragraphs.read().map_err(|e| e.to_string())?;
        paragraphs.len()
    };

    let pos = start_para.unwrap_or_else(|| {
        *state
            .current_position
            .read()
            .map_err(|e| e.to_string())
            .unwrap()
    });

    if pos >= len {
        *state
            .mode
            .write()
            .map_err(|e| e.to_string())? = Mode::View;
        return stop_reading(app, state).await;
    }

    *state
        .current_position
        .write()
        .map_err(|e| e.to_string())? = pos;
    *state
        .mode
        .write()
        .map_err(|e| e.to_string())? = Mode::Reader;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = update_media_session(&app).await;

    read_next_para(app, state.clone()).await?;

    Ok(())
}

async fn read_next_para(app: AppHandle, state: State<'_, SpeakBarState>) -> Result<(), String> {
    let should_stop = {
        let mode = state.mode.read().map_err(|e| e.to_string())?;
        let positions = state.paragraphs.read().map_err(|e| e.to_string())?;
        let pos = *state.current_position.read().map_err(|e| e.to_string())?;
        *mode != Mode::View && pos < positions.len()
    };

    if !should_stop {
        stop_reading_internal(&app, state).await?;
        return Ok(());
    }

    let pos = *state.current_position.read().map_err(|e| e.to_string())?;
    let rate = *state.rate.read().map_err(|e| e.to_string())?;
    let voice_id = state.voice_id.read().map_err(|e| e.to_string())?.clone();
    let text = {
        let positions = state.paragraphs.read().map_err(|e| e.to_string())?;
        positions[pos].clone()
    };

    let mode = state
        .mode
        .read()
        .map_err(|e| e.to_string())?
        .clone();
    app.emit(
        "speakbar:state-changed",
        StateChanged {
            position: Some(pos),
            mode: mode.to_frontend(),
        },
    )
    .map_err(|e| e.to_string())?;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = update_media_session(&app).await;

    let speak_req = tauri_plugin_tts::SpeakRequest {
        text,
        rate,
        voice_id,
        pitch: 1.0,
        volume: 1.0,
        language: None,
        queue_mode: tauri_plugin_tts::QueueMode::Flush,
    };

    if let Err(e) = app.tts().speak(speak_req) {
        app.emit(
            "speakbar:state-changed",
            StateChanged {
                position: None,
                mode: Mode::View,
            },
        )
        .map_err(|e| e.to_string())?;
        return Err(e.to_string());
    }

    Ok(())
}

async fn stop_reading_internal(
    app: &AppHandle,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    *state
        .mode
        .write()
        .map_err(|e| e.to_string())? = Mode::View;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = app.media_session().clear();

    app.emit(
        "speakbar:state-changed",
        StateChanged {
            position: None,
            mode: Mode::View,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
async fn update_media_session(app: &AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<SpeakBarState>() {
        let mode = state
            .mode
            .read()
            .map_err(|e| e.to_string())?
            .clone();
        let is_playing = mode != Mode::View;
        let title = state.title.read().map_err(|e| e.to_string())?.clone();
        let title = if title.is_empty() {
            "Untitled".to_string()
        } else {
            title
        };
        let pos = *state.current_position.read().map_err(|e| e.to_string())?;
        let rate = *state.rate.read().map_err(|e| e.to_string())?;
        let total = *state
            .total_duration
            .read()
            .map_err(|e| e.to_string())?;
        let position = {
            let durations = state
                .cumulative_durations
                .read()
                .map_err(|e| e.to_string())?;
            if pos > 0 && pos <= durations.len() {
                durations[pos - 1] / rate as f64
            } else {
                0.0
            }
        };

        app.media_session()
            .update_state(MediaState {
                title: Some(title),
                duration: Some(total / rate as f64),
                position: Some(position),
                playback_speed: Some(rate as f64),
                is_playing: Some(is_playing),
                artist: Some("estimated time".into()),
                can_prev: Some(false),
                can_next: Some(false),
                can_seek: Some(true),
                ..Default::default()
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_reading(app: AppHandle, state: State<'_, SpeakBarState>) -> Result<(), String> {
    let _ = app.tts().stop();
    stop_reading_internal(&app, state).await
}

#[tauri::command]
pub async fn change_rate(
    rate: f32,
    #[allow(unused_variables)] app: AppHandle,
) -> Result<(), String> {
    if let Some(state) = app.try_state::<SpeakBarState>() {
        *state.rate.write().map_err(|e| e.to_string())? = rate;

        #[cfg(any(target_os = "android", target_os = "ios"))]
        let _ = update_media_session(&app).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_read_state(state: State<'_, SpeakBarState>) -> Result<ReadState, String> {
    let mode = state
        .mode
        .read()
        .map_err(|e| e.to_string())?
        .clone();
    let position = *state.current_position.read().map_err(|e| e.to_string())?;

    Ok(ReadState { mode: mode.to_frontend(), position })
}

#[tauri::command]
pub async fn set_voice_id(
    voice_id: Option<String>,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    *state
        .voice_id
        .write()
        .map_err(|e| e.to_string())? = voice_id;
    Ok(())
}

#[tauri::command]
pub async fn cleanup_reading(
    app: AppHandle,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    let _ = app.tts().stop();

    *state
        .paragraphs
        .write()
        .map_err(|e| e.to_string())? = Vec::new();
    *state
        .title
        .write()
        .map_err(|e| e.to_string())? = String::new();
    *state
        .current_position
        .write()
        .map_err(|e| e.to_string())? = 0;
    *state
        .mode
        .write()
        .map_err(|e| e.to_string())? = Mode::View;
    *state
        .cumulative_durations
        .write()
        .map_err(|e| e.to_string())? = Vec::new();
    *state
        .total_duration
        .write()
        .map_err(|e| e.to_string())? = 0.0;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = app.media_session().clear();

    for id in state
        .tts_listener_ids
        .read()
        .map_err(|e| e.to_string())?
        .iter()
    {
        app.unlisten(*id);
    }
    state
        .tts_listener_ids
        .write()
        .map_err(|e| e.to_string())?
        .clear();

    Ok(())
}
