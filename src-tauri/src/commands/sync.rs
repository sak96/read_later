use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub count_processed: usize,
    pub total_count: usize,
}

#[tauri::command]
pub async fn sync_articles<R: tauri::Runtime>(
    _app_handle: tauri::AppHandle<R>,
    _db_instances: tauri::State<'_, tauri_plugin_sql::DbInstances>,
    _progress_channel: tauri::ipc::Channel<SyncProgress>,
) -> Result<(), String> {
    Ok(())
}
