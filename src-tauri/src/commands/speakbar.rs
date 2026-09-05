use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use tauri_plugin_tts::TtsExt;

use crate::error::TauriError;

#[cfg(any(target_os = "android", target_os = "ios"))]
use tauri_plugin_media_session::{MediaSessionExt, MediaState};

#[cfg(any(target_os = "android", target_os = "ios"))]
use anyhow::Error;

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
    #[must_use]
    pub fn from_is_playing(is_playing: bool) -> Self {
        if is_playing { Self::Reader } else { Self::View }
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

fn write_lock<'a, T>(
    lock: &'a RwLock<T>,
    name: &str,
) -> Result<std::sync::RwLockWriteGuard<'a, T>> {
    lock.write()
        .map_err(|_| anyhow::anyhow!("failed to acquire {name} write lock"))
}

fn read_lock<'a, T>(lock: &'a RwLock<T>, name: &str) -> Result<std::sync::RwLockReadGuard<'a, T>> {
    lock.read()
        .map_err(|_| anyhow::anyhow!("failed to acquire {name} read lock"))
}

#[tauri::command]
pub async fn init_reading(
    app: AppHandle,
    rate: f32,
    title: String,
    paragraphs: Vec<String>,
    state: State<'_, SpeakBarState>,
) -> Result<(), TauriError> {
    init_reading_inner(app, rate, title, paragraphs, state)
        .await
        .map_err(TauriError::from)
}

async fn init_reading_inner(
    app: AppHandle,
    rate: f32,
    title: String,
    paragraphs: Vec<String>,
    state: State<'_, SpeakBarState>,
) -> Result<()> {
    let processed = super::pronunciation::apply_pronunciation_rules(&app, paragraphs)
        .await
        .context("failed to apply pronunciation rules")?;

    *write_lock(&state.paragraphs, "paragraphs").context("failed to update paragraphs")? =
        processed;

    *write_lock(&state.title, "title").context("failed to update title")? = title;

    *write_lock(&state.rate, "rate").context("failed to update rate")? = rate;

    *write_lock(&state.current_position, "current position")
        .context("failed to reset current position")? = 0;

    let listener_finish = {
        let app_ = app.clone();

        app.listen("tts://speech:finish", move |_event: tauri::Event| {
            let app = app_.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(state) = app.try_state::<SpeakBarState>() {
                    let result: Result<()> = async {
                        {
                            let mut position =
                                write_lock(&state.current_position, "current position")
                                    .context("failed to update current position")?;

                            *position = position.saturating_add(1);
                        }

                        // current_position was already advanced above,
                        // so let start_reading_inner use the new position.
                        start_reading_inner(&app, None, &state)
                            .await
                            .context("failed to start reading after speech finished")?;

                        Ok(())
                    }
                    .await;

                    if let Err(error) = result {
                        eprintln!("TTS finish handler failed: {error:#}");
                    }
                }
            });
        })
    };

    let listener_error = {
        let app_ = app.clone();

        app.listen("tts://speech:error", move |_event: tauri::Event| {
            let app = app_.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(state) = app.try_state::<SpeakBarState>() {
                    if let Err(error) = stop_reading_inner(&app, &state).await {
                        eprintln!("failed to stop reading after TTS error: {error:#}");
                    }
                }
            });
        })
    };

    let listener_interrupted = {
        let app_ = app.clone();

        app.listen("tts://speech:interrupted", move |_event: tauri::Event| {
            let app = app_.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(state) = app.try_state::<SpeakBarState>() {
                    if let Err(error) = stop_reading_inner(&app, &state).await {
                        eprintln!("failed to stop reading after TTS interruption: {error:#}");
                    }
                }
            });
        })
    };

    let mut listener_ids = write_lock(&state.tts_listener_ids, "TTS listener IDs")
        .context("failed to update TTS listener IDs")?;

    listener_ids.clear();
    listener_ids.extend([listener_finish, listener_error, listener_interrupted]);

    Ok(())
}

async fn start_reading_inner(
    app: &AppHandle,
    start_para: Option<usize>,
    state: &State<'_, SpeakBarState>,
) -> Result<()> {
    let len = read_lock(&state.paragraphs, "paragraphs")
        .context("failed to read paragraphs")?
        .len();

    let pos = match start_para {
        Some(pos) => pos,
        None => *read_lock(&state.current_position, "current position")
            .context("failed to read current position")?,
    };

    if pos >= len {
        *write_lock(&state.is_playing, "is playing").context("failed to update playing state")? =
            false;

        stop_reading_inner(app, state)
            .await
            .context("failed to stop reading")?;

        return Ok(());
    }

    *write_lock(&state.current_position, "current position")
        .context("failed to update current position")? = pos;

    *write_lock(&state.is_playing, "is playing").context("failed to update playing state")? = true;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    update_media_session(app)
        .await
        .context("failed to update media session")?;

    read_next_para(app, state)
        .await
        .context("failed to read next paragraph")?;

    Ok(())
}

async fn read_next_para(app: &AppHandle, state: &State<'_, SpeakBarState>) -> Result<()> {
    // Keep all RwLock guards inside this scope. Nothing returned from this
    // block contains a lock guard, so no non-Send guard can live across await.
    let reading_data = {
        let is_playing =
            *read_lock(&state.is_playing, "is playing").context("failed to read playing state")?;

        let pos = *read_lock(&state.current_position, "current position")
            .context("failed to read current position")?;

        let paragraphs =
            read_lock(&state.paragraphs, "paragraphs").context("failed to read paragraphs")?;

        if !is_playing || pos >= paragraphs.len() {
            None
        } else {
            let text = paragraphs
                .get(pos)
                .cloned()
                .context("paragraph position is out of bounds")?;

            let rate = *read_lock(&state.rate, "rate").context("failed to read rate")?;

            let voice_id = read_lock(&state.voice_id, "voice ID")
                .context("failed to read voice ID")?
                .clone();

            Some((pos, text, rate, voice_id))
        }
    };

    let Some((pos, text, rate, voice_id)) = reading_data else {
        stop_reading_internal(app, state).context("failed to stop reading internally")?;

        return Ok(());
    };

    let mode = Mode::from_is_playing(true);

    app.emit(
        "speakbar:state-changed",
        StateChanged {
            position: Some(pos),
            mode,
        },
    )
    .context("failed to emit speakbar state-changed event")?;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    update_media_session(app)
        .await
        .context("failed to update media session")?;

    let speak_req = tauri_plugin_tts::SpeakRequest {
        text,
        rate,
        voice_id,
        pitch: 1.0,
        volume: 1.0,
        language: None,
        queue_mode: tauri_plugin_tts::QueueMode::Flush,
    };

    if let Err(error) = app.tts().speak(speak_req) {
        app.emit(
            "speakbar:state-changed",
            StateChanged {
                position: None,
                mode: Mode::View,
            },
        )
        .context("failed to emit state after TTS error")?;

        return Err(error).context("failed to start TTS speech");
    }

    Ok(())
}

fn stop_reading_internal(app: &AppHandle, state: &State<'_, SpeakBarState>) -> Result<()> {
    *write_lock(&state.is_playing, "is playing").context("failed to update playing state")? = false;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.media_session()
        .clear()
        .map_err(|e| anyhow::anyhow!("failed to clear media session: {e}"))?;

    app.emit(
        "speakbar:state-changed",
        StateChanged {
            position: None,
            mode: Mode::View,
        },
    )
    .context("failed to emit speakbar state-changed event")?;

    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
async fn update_media_session(app: &AppHandle) -> Result<(), Error> {
    let Some(state) = app.try_state::<SpeakBarState>() else {
        return Ok(());
    };

    let is_playing = {
        let guard =
            read_lock(&state.is_playing, "is playing").context("failed to read playing state")?;

        *guard
    };

    let title = {
        let guard = read_lock(&state.title, "title").context("failed to read title")?;

        guard.clone()
    };

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
        .map_err(|e| anyhow::anyhow!("failed to update media session: {e}"))?;

    Ok(())
}

#[tauri::command]
pub async fn start_reading(
    app: AppHandle,
    start_para: Option<usize>,
    state: State<'_, SpeakBarState>,
) -> Result<(), TauriError> {
    start_reading_inner(&app, start_para, &state)
        .await
        .map_err(TauriError::from)
}

#[tauri::command]
pub async fn stop_reading(
    app: AppHandle,
    state: State<'_, SpeakBarState>,
) -> Result<(), TauriError> {
    stop_reading_inner(&app, &state)
        .await
        .map_err(TauriError::from)
}

async fn stop_reading_inner(app: &AppHandle, state: &State<'_, SpeakBarState>) -> Result<()> {
    app.tts().stop().context("failed to stop TTS")?;

    stop_reading_internal(app, state).context("failed to stop reading internally")?;

    Ok(())
}

#[tauri::command]
pub async fn change_rate(rate: f32, state: State<'_, SpeakBarState>) -> Result<(), TauriError> {
    *write_lock(&state.rate, "rate")
        .context("failed to update rate")
        .map_err(TauriError::from)? = rate;

    Ok(())
}

#[tauri::command]
pub async fn get_read_state(state: State<'_, SpeakBarState>) -> Result<ReadState, TauriError> {
    let is_playing = *read_lock(&state.is_playing, "is playing")
        .context("failed to read playing state")
        .map_err(TauriError::from)?;

    let position = *read_lock(&state.current_position, "current position")
        .context("failed to read current position")
        .map_err(TauriError::from)?;

    Ok(ReadState {
        mode: Mode::from_is_playing(is_playing),
        position,
    })
}

#[tauri::command]
pub async fn set_voice_id(
    voice_id: Option<String>,
    state: State<'_, SpeakBarState>,
) -> Result<(), TauriError> {
    *write_lock(&state.voice_id, "voice ID")
        .context("failed to update voice ID")
        .map_err(TauriError::from)? = voice_id;

    Ok(())
}

#[tauri::command]
pub async fn cleanup_reading(
    app: AppHandle,
    state: State<'_, SpeakBarState>,
) -> Result<(), TauriError> {
    app.tts()
        .stop()
        .context("failed to stop TTS during cleanup")
        .map_err(TauriError::from)?;

    *write_lock(&state.paragraphs, "paragraphs")
        .context("failed to clear paragraphs")
        .map_err(TauriError::from)? = Vec::new();

    *write_lock(&state.title, "title")
        .context("failed to clear title")
        .map_err(TauriError::from)? = String::new();

    *write_lock(&state.current_position, "current position")
        .context("failed to reset current position")
        .map_err(TauriError::from)? = 0;

    *write_lock(&state.is_playing, "is playing")
        .context("failed to reset playing state")
        .map_err(TauriError::from)? = false;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.media_session()
        .clear()
        .map_err(|e| TauriError::from(anyhow::anyhow!("failed to clear media session: {e}")))?;

    let listener_ids = read_lock(&state.tts_listener_ids, "TTS listener IDs")
        .context("failed to read TTS listener IDs")
        .map_err(TauriError::from)?
        .clone();

    for id in listener_ids {
        app.unlisten(id);
    }

    write_lock(&state.tts_listener_ids, "TTS listener IDs")
        .context("failed to clear TTS listener IDs")
        .map_err(TauriError::from)?
        .clear();

    Ok(())
}
