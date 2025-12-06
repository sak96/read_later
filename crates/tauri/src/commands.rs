use crate::models::DB_URL;
use crate::parse::process_html;
pub use import_export::*;
use readabilityrs::Readability;
use shared::models::*;
use sqlx::{query, query_as, query_scalar};
use std::io::{BufReader, BufWriter};
use tauri::State;
use tauri_plugin_http::reqwest;
use tauri_plugin_sql::DbInstances;

#[tauri::command]
pub async fn get_articles(
    db_instances: State<'_, DbInstances>,
    offset: usize,
) -> Result<Vec<ArticleEntry>, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => query_as::<_, ArticleEntry>(
            "SELECT id, title, datetime(created_at, 'localtime') created_at FROM articles ORDER BY created_at DESC limit $1, 100",
        )
        .bind(offset.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub async fn get_article(id: i32, db_instances: State<'_, DbInstances>) -> Result<Article, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let mut article =
                query_as::<_, Article>("SELECT id, title, body, url FROM articles WHERE id = ?")
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            if article.title.is_empty() {
                let html = reqwest::get(&article.url)
                    .await
                    .map_err(|e| e.to_string())?
                    .text()
                    .await
                    .map_err(|e| e.to_string())?;

                // Readability is not send.
                let article_data = {
                    Readability::new(&html, Some(&article.url), None)
                        .map_err(|e| format!("Failed to parse: {:?}", e))?
                        .parse()
                        .ok_or("Failed to extract article")?
                };

                let title = match article_data.title {
                    Some(v) if v.is_empty() => "Untitled".into(),
                    None => "Untitled".into(),
                    Some(v) => v,
                };
                let body = article_data.content.unwrap_or_default();

                // could be update
                article = query_as::<_, Article>(
                    r#"INSERT INTO articles
                        (id, title, body, url)
                        VALUES ($1, $2, $3, $4)
                      ON CONFLICT(id) do update SET
                        title = $2, body = $3, url = $3
                      RETURNING id, title, body, created_at, url"#,
                )
                .bind(article.id)
                .bind(title)
                .bind(body)
                .bind(&article.url)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            article.body = process_html(&article.body, 1000, &mut 0);
            Ok(article)
        }
    }
}

#[tauri::command]
pub async fn add_article(
    url: String,
    db_instances: State<'_, DbInstances>,
) -> Result<Article, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let article = query_as::<_, Article>(
                "INSERT INTO articles (title, body, url) VALUES ('', '', $1) RETURNING id, title, body, created_at, url",
            )
            .bind(url)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(article)
        }
    }
}

#[tauri::command]
pub async fn delete_article(id: i32, db_instances: State<'_, DbInstances>) -> Result<u64, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let result = query("DELETE FROM articles WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

            Ok(result.rows_affected())
        }
    }
}

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
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query("INSERT into settings (name, value, default_value) values($1, $2, '') ON CONFLICT(name) do update SET value = $2, default_value = ''")
                .bind(name)
                .bind(value)
                .execute(pool)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    }
}

#[cfg(not(target_os = "android"))]
mod import_export {
    use super::*;
    use std::fs::File;
    use tauri_plugin_dialog::DialogExt;
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
                add_article(url, db_instances.clone()).await?;
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
}

#[cfg(target_os = "android")]
mod import_export {
    use super::*;
    use tauri_plugin_android_fs::AndroidFsExt;
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
                add_article(url, db_instances.clone()).await?;
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
}
