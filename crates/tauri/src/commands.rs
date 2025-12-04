use crate::models::*;
use crate::parse::process_html;
use readabilityrs::Readability;
use sqlx::{query, query_as};
use tauri::State;
use tauri_plugin_http::reqwest;
use tauri_plugin_sql::DbInstances;

#[tauri::command]
pub async fn get_articles(db_instances: State<'_, DbInstances>) -> Result<Vec<Article>, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        // TODO: Paginate
        tauri_plugin_sql::DbPool::Sqlite(pool) => query_as::<_, Article>(
            "SELECT id, title, body, datetime(created_at, 'localtime') created_at, url FROM articles ORDER BY created_at DESC",
        )
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
            let mut article = query_as::<_, Article>(
                "SELECT id, title, body, created_at, url FROM articles WHERE id = ?",
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
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
            let html = reqwest::get(&url)
                .await
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())?;

            // Readability is not send.
            let article_data = {
                Readability::new(&html, Some(&url), None)
                    .map_err(|e| format!("Failed to parse: {:?}", e))?
                    .parse()
                    .ok_or("Failed to extract article")?
            };

            let title = article_data.title.unwrap_or_else(|| "Untitled".to_string());
            let body = article_data.content.unwrap_or_default();

            let article = query_as::<_, Article>(
                "INSERT INTO articles (title, body, url) VALUES (?, ?, ?) RETURNING id, title, body, created_at, url",
            )
            .bind(title)
            .bind(body)
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
