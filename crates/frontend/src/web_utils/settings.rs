use crate::web_utils::{invoke_no_parse_log_error, invoke_parse_log_error};

pub async fn get_setting(name: &str) -> Option<String> {
    invoke_parse_log_error("get_setting", &Some(serde_json::json!({"name": name}))).await
}

pub async fn set_setting(name: &str, value: &str) {
    invoke_no_parse_log_error(
        "set_setting",
        &Some(serde_json::json!({ "name": name, "value": value })),
    )
    .await;
}
