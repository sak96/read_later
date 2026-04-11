use crate::models::*;
use sqlx::query_scalar;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_sql::DbInstances;

#[tauri::command]
pub async fn pick_import_file(
    app: tauri::AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    if let Some(file_path) = app.dialog().file().blocking_pick_file() {
        let path = file_path.as_path().ok_or("could not get a path")?;
        let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
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
    if let Some(file_path) = app
        .dialog()
        .file()
        .add_filter("JSON Files", &["json"])
        .blocking_save_file()
    {
        let path = file_path.as_path().ok_or("could not get a path")?;
        let file = File::create(path).map_err(|e| format!("could not create path: {e}"))?;
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
