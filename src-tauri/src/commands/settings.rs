use crate::{error::TauriError, models::DB_URL};
use anyhow::{Context, Error, Result};
use sqlx::{query, query_as};
use tauri::State;
use tauri_plugin_sql::DbInstances;

async fn get_setting_inner(
    name: String,
    db_instances: &State<'_, DbInstances>,
) -> Result<String, Error> {
    let instances = db_instances.0.read().await;

    let db = instances.get(DB_URL).context("database is not loaded")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let result: (String,) = query_as("SELECT value FROM settings WHERE name = ?")
                .bind(name)
                .fetch_one(pool)
                .await
                .context("failed to fetch setting")?;

            Ok(result.0)
        }
    }
}

async fn set_setting_inner(
    name: String,
    value: String,
    db_instances: &State<'_, DbInstances>,
) -> Result<(), Error> {
    let instances = db_instances.0.write().await;

    let db = instances.get(DB_URL).context("database is not loaded")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query(
                r"
                INSERT INTO settings (name, value, default_value)
                VALUES ($1, $2, '')
                ON CONFLICT(name) DO UPDATE SET
                    value = $2,
                    default_value = ''
                ",
            )
            .bind(name)
            .bind(value)
            .execute(pool)
            .await
            .context("failed to set setting")?;

            Ok(())
        }
    }
}

async fn delete_setting_inner(
    name: String,
    db_instances: &State<'_, DbInstances>,
) -> Result<(), Error> {
    let instances = db_instances.0.read().await;

    let db = instances.get(DB_URL).context("database is not loaded")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query("DELETE FROM settings WHERE name = ?")
                .bind(name)
                .execute(pool)
                .await
                .context("failed to delete setting")?;

            Ok(())
        }
    }
}

#[tauri::command]
pub async fn get_setting(
    name: String,
    db_instances: State<'_, DbInstances>,
) -> Result<String, TauriError> {
    get_setting_inner(name, &db_instances)
        .await
        .map_err(TauriError::from)
}

#[tauri::command]
pub async fn set_setting(
    name: String,
    value: String,
    db_instances: State<'_, DbInstances>,
) -> Result<(), TauriError> {
    set_setting_inner(name, value, &db_instances)
        .await
        .map_err(TauriError::from)
}

#[tauri::command]
pub async fn delete_setting(
    name: String,
    db_instances: State<'_, DbInstances>,
) -> Result<(), TauriError> {
    delete_setting_inner(name, &db_instances)
        .await
        .map_err(TauriError::from)
}
