use crate::web_utils::{invoke_no_parse, invoke_no_parse_log_error, invoke_parse_log_error};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct TTSVoice {
    pub id: String,
    pub name: String,
    pub lang: String,
    #[serde(default)]
    pub disabled: bool,
}

impl TTSVoice {
    pub fn label(&self) -> String {
        format!("{}_{}", self.name, self.lang)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVoicesResponse {
    pub voices: Vec<TTSVoice>,
}

// TTS plugin connectors
pub async fn speak(text: String, rate: f32) {
    invoke_no_parse_log_error(
        "plugin:tts|speak",
        &Some(serde_json::json!({"args": {"text": text, "rate": rate}})),
    )
    .await;
}

pub async fn stop_speak() {
    invoke_no_parse_log_error("plugin:tts|stop", &None).await;
}

pub async fn get_voices() -> Option<GetVoicesResponse> {
    invoke_parse_log_error::<GetVoicesResponse>("plugin:tts|get_all_voices", &None).await
}

pub async fn set_voice(id: &str) -> bool {
    invoke_no_parse(
        "plugin:tts|set_voice",
        &Some(serde_json::json!({"voice": id})),
    )
    .await
    .is_ok()
}
