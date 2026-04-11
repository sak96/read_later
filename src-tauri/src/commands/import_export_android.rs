use crate::models::*;
use sqlx::query_scalar;
use std::io::{BufReader, BufWriter};
use tauri::State;
use tauri_plugin_android_fs::AndroidFsExt;
use tauri_plugin_sql::DbInstances;

#[tauri::command]
pub async fn pick_import_file(
    app: tauri::AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let api = app.android_fs();
    if let Ok(Some(file_path)) =
        app.android_fs()
            .file_picker()
            .pick_file(None, &["application/json"], true)
    {
        let file: std::fs::File = api
            .open_file_readable(&file_path)
            .map_err(|err| err.to_string())?;
        let reader = BufReader::new(file);
        for url in serde_json::from_reader::<_, Vec<String>>(reader)
            .map_err(|e| format!("Failed to parse file: {e}"))?
        {
            crate::commands::add_article(url, db_instances.clone()).await?;
        }
        Ok(())
    } else {
        Err("No file selected".into())
    }
}

#[tauri::command]
pub async fn pick_export_file(
    app: tauri::AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let api = app.android_fs();
    if let Ok(Some(file_path)) = app.android_fs().file_picker().save_file(
        None,
        "read_later.json",
        Some("application/json"),
        true,
    ) {
        let file: std::fs::File = api
            .open_file_writable(&file_path)
            .map_err(|err| err.to_string())?;
        let writer = BufWriter::new(file);
        let instances = db_instances.0.read().await;
        let db = instances.get(DB_URL).ok_or("db not loaded")?;
        let urls = match db {
            tauri_plugin_sql::DbPool::Sqlite(pool) => {
                query_scalar::<_, String>("SELECT url FROM articles ORDER BY created_at")
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())?
            }
        };
        serde_json::to_writer(writer, &urls)
            .map_err(|e| format!("Failed to write to file: {e}"))?;
        Ok(())
    } else {
        Err("No save location selected".into())
    }
}
