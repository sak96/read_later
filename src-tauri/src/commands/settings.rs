use crate::models::DB_URL;
use sqlx::{query, query_as};
use tauri::State;
use tauri_plugin_sql::DbInstances;

#[tauri::command]
pub async fn get_setting(
    name: String,
    db_instances: State<'_, DbInstances>,
) -> Result<String, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            // We only select the 'value' column
            let result: (String,) = query_as("SELECT value FROM settings WHERE name = ?")
                .bind(name)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

            Ok(result.0)
        }
    }
}

#[tauri::command]
pub async fn set_setting(
    name: String,
    value: String,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let instances = db_instances.0.write().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query(r"
                INSERT INTO settings (name, value, default_value)
                VALUES ($1, $2, '')
                ON CONFLICT(name) DO UPDATE SET
                    value = $2,
                    default_value = ''
            ")
                .bind(name)
                .bind(value)
                .execute(pool)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    }
}

#[tauri::command]
pub async fn delete_setting(
    name: String,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query("DELETE FROM settings WHERE name = ?")
                .bind(name)
                .execute(pool)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    }
}
