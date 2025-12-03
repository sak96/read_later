use crate::web_utils::invoke_no_parse_log_error;

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
