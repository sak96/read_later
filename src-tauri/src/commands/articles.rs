use crate::models::*;
use crate::parse::{build_snippet, process_html};
use readabilityrs::Readability;
use sqlx::{query, query_as};
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
            let rows = sqlx::query_as::<_, ArticleEntryRow>(
                r#"
                SELECT id, url, title, text_content,
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

            let articles: Vec<ArticleEntry> = rows
                .into_iter()
                .map(|row| ArticleEntry {
                    id: row.id,
                    url: row.url,
                    title: row.title,
                    snippet: build_snippet(&row.text_content, query),
                    created_at: row.created_at,
                })
                .collect();

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
pub async fn get_article_count(db_instances: State<'_, DbInstances>) -> Result<i64, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles")
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

            Ok(count.0)
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
