use crate::models::*;
use sqlx::{query, query_as, query_scalar};
use tauri::{State, ipc::Channel};
use tauri_plugin_sql::DbInstances;

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
                ) and is_deleted == 0
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

use crate::fetcher::{new_fetcher, Fetcher, FetcherMode};
use crate::parse::{build_snippet, process_html};
use readabilityrs::Readability;

async fn fetch_parse_update_article(
    article_url: &str,
    fetcher: &mut dyn Fetcher,
    on_progress: &Channel<FetchProgress>,
) -> Result<(String, String, String), String> {
    on_progress
        .send(FetchProgress::Downloading(article_url.to_string()))
        .map_err(|e| e.to_string())?;

    let html = fetcher.fetch().await?;

    let options = readabilityrs::ReadabilityOptions::builder()
        .remove_title_from_content(true)
        .build();
    let article_data = Readability::new(&html, Some(article_url), Some(options))
        .map_err(|e| format!("Failed to parse: {:?}", e))?
        .parse()
        .ok_or("Failed to extract article")?;

    let title = match article_data.title {
        Some(v) if v.is_empty() => "Untitled".into(),
        None => "Untitled".into(),
        Some(v) => v,
    };
    let body = article_data.content.unwrap_or_default();
    let text_content = article_data.text_content.unwrap_or_default();

    Ok((title, body, text_content))
}

#[tauri::command]
pub async fn get_article(
    id: i32,
    db_instances: State<'_, DbInstances>,
    on_progress: Channel<FetchProgress>,
    app: tauri::AppHandle,
) -> Result<Article, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let mut article = query_as::<_, Article>(
                r#"
                SELECT id, title, body, url
                FROM articles
                WHERE is_deleted == 0 AND id = ?
                "#,
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            if article.title.is_empty() {
                let mode = sqlx::query_as::<_, (String,)>(
                    "SELECT value FROM settings WHERE name = 'fetcher_mode'",
                )
                .fetch_one(pool)
                .await
                .ok()
                .map(|r| r.0)
                .and_then(|v| FetcherMode::from_str(&v))
                .unwrap_or(FetcherMode::Html);

                let mut fetcher = new_fetcher(&app, &article.url, mode)?;

                article = match fetch_parse_update_article(
                    &article.url,
                    &mut *fetcher,
                    &on_progress,
                )
                .await
                {
                    Ok((title, body, text_content)) => query_as::<_, Article>(
                        r#"
                                UPDATE articles
                                SET title = $2, body = $3, url = $4, text_content = $5
                                WHERE id = $1
                                RETURNING id, title, body, created_at, url
                            "#,
                    )
                    .bind(article.id)
                    .bind(title)
                    .bind(body)
                    .bind(&article.url)
                    .bind(text_content)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string()),
                    Err(e) => {
                        let _ = query(
                            "UPDATE articles SET is_deleted = 1, title = '', body = '', text_content = '' WHERE id = ?",
                        )
                        .bind(id)
                        .execute(pool)
                        .await;
                        Err(e)
                    }
                }?;
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
                r#"
                INSERT INTO articles (title, body, url, updated_at)
                VALUES ('', '', $1, datetime('now'))
                ON CONFLICT(url) DO UPDATE SET
                    is_deleted = 0,
                    updated_at = datetime('now')
                RETURNING id, title, body, created_at, url
                "#,
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
            let count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM articles WHERE is_deleted == 0")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            Ok(count.0)
        }
    }
}

#[tauri::command]
pub async fn refresh_article(id: i32, db_instances: State<'_, DbInstances>) -> Result<(), String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query(
                r#"
                UPDATE articles
                SET title = '', body = '', text_content = '', updated_at = datetime('now')
                WHERE id = ?
            "#,
            )
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn delete_article(id: i32, db_instances: State<'_, DbInstances>) -> Result<u64, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let result = query(r#"
                UPDATE articles
                SET is_deleted = 1, title = '', body = '', text_content = '', updated_at = datetime('now')
                WHERE id = ? AND is_deleted = 0
            "#)
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

            Ok(result.rows_affected())
        }
    }
}

#[tauri::command]
pub async fn pick_import_file(
    app: tauri::AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let urls: Vec<String> = crate::file_helpers::pick_and_read_json(&app)?;
    for url in urls {
        add_article(url, db_instances.clone()).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pick_export_file(
    app: tauri::AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    let urls = match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => query_scalar::<_, String>(
            "SELECT url FROM articles where is_deleted == 0 ORDER BY created_at",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?,
    };
    crate::file_helpers::pick_and_write_json(&app, &urls, "read_later.json")
}
