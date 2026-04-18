use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use tauri_plugin_tts::TtsExt;

#[cfg(any(target_os = "android", target_os = "ios"))]
use tauri_plugin_media_session::{MediaSessionExt, MediaState};

#[derive(Debug, Deserialize, Default, PartialEq)]
pub enum MediaAction {
    #[default]
    Stop,
    Play,
    Pause,
}

#[derive(Debug, Deserialize, Default)]
pub struct MediaActionEvent {
    #[serde(default)]
    pub action: Option<MediaAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Mode {
    #[serde(rename = "view")]
    View,
    #[serde(rename = "reader")]
    Reader,
}

impl Mode {
    pub fn from_is_playing(is_playing: bool) -> Self {
        if is_playing { Mode::Reader } else { Mode::View }
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
    pub is_playing: RwLock<bool>,
    pub tts_listener_ids: RwLock<Vec<u32>>,
}

impl Default for SpeakBarState {
    fn default() -> Self {
        Self {
            paragraphs: RwLock::new(Vec::new()),
            title: RwLock::new(String::new()),
            current_position: RwLock::new(0),
            rate: RwLock::new(1.0),
            voice_id: RwLock::new(None),
            is_playing: RwLock::new(false),
            tts_listener_ids: RwLock::new(Vec::new()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadState {
    pub mode: Mode,
    pub position: usize,
}

#[tauri::command]
pub async fn init_reading(
    app: AppHandle,
    rate: f32,
    title: String,
    paragraphs: Vec<String>,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    *state.paragraphs.write().map_err(|e| e.to_string())? = paragraphs;
    *state.title.write().map_err(|e| e.to_string())? = title;
    *state.rate.write().map_err(|e| e.to_string())? = rate;
    *state.current_position.write().map_err(|e| e.to_string())? = 0;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = update_media_session(&app).await;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.tts()
        .setup_event_relay(&app.clone())
        .map_err(|e| e.to_string())?;

    let listener_finish = {
        let app_clone = app.clone();
        app_clone.clone().listen("tts://speech:finish", {
            move |_event: tauri::Event| {
                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app.try_state::<SpeakBarState>() {
                        let pos = *state
                            .current_position
                            .read()
                            .map_err(|e| e.to_string())
                            .unwrap();
                        *state
                            .current_position
                            .write()
                            .map_err(|e| e.to_string())
                            .unwrap() = pos + 1;
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
                        let app = app.clone();
                        let _ = stop_reading(app, state).await;
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
        *state.is_playing.write().map_err(|e| e.to_string())? = false;
        return stop_reading(app, state).await;
    }

    *state.current_position.write().map_err(|e| e.to_string())? = pos;
    *state.is_playing.write().map_err(|e| e.to_string())? = true;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = update_media_session(&app).await;

    read_next_para(app, state.clone()).await?;

    Ok(())
}

async fn read_next_para(app: AppHandle, state: State<'_, SpeakBarState>) -> Result<(), String> {
    let should_stop = {
        let is_playing = *state.is_playing.read().map_err(|e| e.to_string())?;
        let positions = state.paragraphs.read().map_err(|e| e.to_string())?;
        let pos = *state.current_position.read().map_err(|e| e.to_string())?;
        is_playing && pos < positions.len()
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

    let is_playing = *state.is_playing.read().map_err(|e| e.to_string())?;
    let mode = Mode::from_is_playing(is_playing);
    app.emit(
        "speakbar:state-changed",
        StateChanged {
            position: Some(pos),
            mode,
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
    *state.is_playing.write().map_err(|e| e.to_string())? = false;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = update_media_session(&app).await;

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
        let is_playing = *state.is_playing.read().map_err(|e| e.to_string())?;
        let title = state.title.read().map_err(|e| e.to_string())?.clone();
        let title = if title.is_empty() {
            "Untitled".to_string()
        } else {
            title
        };

        app.media_session()
            .update_state(MediaState {
                title: Some(title),
                is_playing: Some(is_playing),
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
pub async fn change_rate(rate: f32, state: State<'_, SpeakBarState>) -> Result<(), String> {
    *state.rate.write().map_err(|e| e.to_string())? = rate;
    Ok(())
}

#[tauri::command]
pub async fn get_read_state(state: State<'_, SpeakBarState>) -> Result<ReadState, String> {
    let is_playing = *state.is_playing.read().map_err(|e| e.to_string())?;
    let position = *state.current_position.read().map_err(|e| e.to_string())?;

    Ok(ReadState {
        mode: Mode::from_is_playing(is_playing),
        position,
    })
}

#[tauri::command]
pub async fn set_voice_id(
    voice_id: Option<String>,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    *state.voice_id.write().map_err(|e| e.to_string())? = voice_id;
    Ok(())
}

#[tauri::command]
pub async fn cleanup_reading(
    app: AppHandle,
    state: State<'_, SpeakBarState>,
) -> Result<(), String> {
    let _ = app.tts().stop();

    *state.paragraphs.write().map_err(|e| e.to_string())? = Vec::new();
    *state.title.write().map_err(|e| e.to_string())? = String::new();
    *state.current_position.write().map_err(|e| e.to_string())? = 0;
    *state.is_playing.write().map_err(|e| e.to_string())? = false;

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
