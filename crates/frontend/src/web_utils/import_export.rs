use crate::web_utils::invoke_no_parse_log_error;

pub async fn import_data() {
    invoke_no_parse_log_error("pick_import_file", &None).await;
}

pub async fn export_data() {
    invoke_no_parse_log_error("pick_export_file", &None).await;
}
