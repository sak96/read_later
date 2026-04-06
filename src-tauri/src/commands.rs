use crate::models::*;
use crate::parse::{build_snippet, process_html};
pub use import_export::*;
use readabilityrs::Readability;
use sqlx::{query, query_as, query_scalar};
use std::io::{BufReader, BufWriter};
use tauri::{State, ipc::Channel};
use tauri_plugin_http::reqwest;
use tauri_plugin_sql::DbInstances;

const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

#[tauri::command]
pub async fn get_articles(
    db_instances: State<'_, DbInstances>,
    offset: usize,
    query: Option<String>,
) -> Result<Vec<ArticleEntry>, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let query: Option<&str> = query.as_ref().filter(|s| s.len() >= 3).map(|s| s.as_str());
            let mut articles = sqlx::query_as::<_, ArticleEntry>(
                r#"
                SELECT id, url, title, text_content as snippet,
                       datetime(created_at, 'localtime') as created_at
                FROM articles
                WHERE (
                    ?1 IS NULL
                    OR LOWER(title) LIKE '%' || LOWER(?1) || '%'
                    OR LOWER(text_content)  LIKE '%' || LOWER(?1) || '%'
                )
                ORDER BY created_at DESC
                LIMIT 100 OFFSET ?2
               "#,
            )
            .bind(query)
            .bind(offset.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            // attach snippet
            for article in &mut articles {
                article.snippet = build_snippet(&article.snippet, query);
            }

            Ok(articles)
        }
    }
}

#[tauri::command]
pub async fn get_article(
    id: i32,
    db_instances: State<'_, DbInstances>,
    on_progress: Channel<FetchProgress>,
) -> Result<Article, String> {
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
                on_progress
                    .send(FetchProgress::Downloading(article.url.to_string()))
                    .map_err(|e| e.to_string())?;
                let client = reqwest::Client::new();
                let html = client
                    .get(&article.url)
                    .header(reqwest::header::USER_AGENT, CHROME_USER_AGENT)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?
                    .text()
                    .await
                    .map_err(|e| e.to_string())?;

                let options = readabilityrs::ReadabilityOptions::builder()
                    .remove_title_from_content(true)
                    .debug(true)
                    .build();
                // Readability is not send.
                let article_data = {
                    Readability::new(&html, Some(&article.url), Some(options))
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
                let text_content = article_data.text_content.unwrap_or_default();

                // could be update
                article = query_as::<_, Article>(
                    "UPDATE articles SET title = $2, body = $3, url = $4, text_content = $5 where id = $1 RETURNING id, title, body, created_at, url",
                )
                .bind(article.id)
                .bind(title)
                .bind(body)
                .bind(&article.url)
                .bind(text_content)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            on_progress
                .send(FetchProgress::Parsing(article.title.to_string()))
                .map_err(|e| e.to_string())?;
            article.body = process_html(&article.body, &article.url);
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
