use crate::error::TauriError;
use crate::fetcher::{FetcherMode, fetch_parse_update_article, new_fetcher};
use crate::models::{Article, ArticleEntry, ArticleEntryRow, DB_URL};
use crate::parse::{build_snippet, process_html};

use anyhow::{Context, Error, Result};
use sqlx::{query, query_as, query_scalar};
use tauri::{Manager, State};
use tauri_plugin_sql::DbInstances;

#[tauri::command]
pub async fn get_articles(
    db_instances: State<'_, DbInstances>,
    offset: usize,
    query: Option<String>,
) -> Result<Vec<ArticleEntry>, TauriError> {
    let instances = db_instances.0.read().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let search_query: Option<&str> =
                query.as_ref().filter(|s| s.len() >= 3).map(String::as_str);

            let rows = query_as::<_, ArticleEntryRow>(
                r"
                SELECT
                    id,
                    url,
                    title,
                    text_content,
                    datetime(created_at, 'localtime') AS created_at
                FROM articles
                WHERE (
                    ?1 IS NULL
                    OR LOWER(title) LIKE '%' || LOWER(?1) || '%'
                    OR LOWER(text_content) LIKE '%' || LOWER(?1) || '%'
                )
                AND is_deleted = 0
                ORDER BY created_at DESC
                LIMIT 100 OFFSET ?2
                ",
            )
            .bind(search_query)
            .bind(offset.to_string())
            .fetch_all(pool)
            .await
            .context("failed to fetch articles from database")?;

            let articles = rows
                .into_iter()
                .map(|row| ArticleEntry {
                    id: row.id,
                    url: row.url,
                    title: row.title,
                    snippet: build_snippet(&row.text_content, search_query),
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
    app: tauri::AppHandle,
) -> Result<Option<Article>, TauriError> {
    let instances = db_instances.0.read().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let article = query_as::<_, Article>(
                r"
                SELECT id, title, body, url
                FROM articles
                WHERE is_deleted = 0
                  AND id = ?
                ",
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .with_context(|| format!("failed to fetch article with id {id}"))?;

            let Some(mut article) = article else {
                return Ok(None);
            };

            if article.title.is_empty() {
                let mode = query_as::<_, (String,)>(
                    "SELECT value FROM settings WHERE name = 'fetcher_mode'",
                )
                .fetch_one(pool)
                .await
                .context("failed to read fetcher mode from settings")
                .ok()
                .and_then(|row| row.0.parse::<FetcherMode>().ok())
                .unwrap_or_default();

                let mut fetcher = new_fetcher(&app, &article.url, mode)
                    .context("failed to create article fetcher")?;

                let article_url = article.url.clone();
                let article_id = article.id;

                tauri::async_runtime::spawn(async move {
                    if let Err(error) =
                        update_article_in_background(app, article_id, article_url, &mut *fetcher)
                            .await
                    {
                        eprintln!("failed to update article in background: {error:#}");
                    }
                });

                return Ok(None);
            }

            article.body = process_html(&article.body, &article.url);

            Ok(Some(article))
        }
    }
}

async fn update_article_in_background(
    app: tauri::AppHandle,
    id: i32,
    url: String,
    fetcher: &mut dyn crate::fetcher::Fetcher,
) -> Result<(), Error> {
    let db_instances = app.state::<DbInstances>();

    let instances = db_instances.0.write().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance for background article update")?;

    let tauri_plugin_sql::DbPool::Sqlite(pool) = db;

    let (title, body, text_content) = fetch_parse_update_article(&url, fetcher)
        .await
        .with_context(|| format!("failed to fetch and parse article: {url}"))?;

    query_as::<_, Article>(
        r"
        UPDATE articles
        SET
            title = $2,
            body = $3,
            url = $4,
            text_content = $5
        WHERE id = $1
        RETURNING id, title, body, created_at, url
        ",
    )
    .bind(id)
    .bind(title)
    .bind(body)
    .bind(&url)
    .bind(text_content)
    .fetch_one(pool)
    .await
    .with_context(|| format!("failed to save fetched article with id {id}"))?;

    Ok(())
}

#[tauri::command]
pub async fn add_article(
    url: String,
    db_instances: State<'_, DbInstances>,
) -> Result<Article, TauriError> {
    let instances = db_instances.0.write().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let article = query_as::<_, Article>(
                r"
                INSERT INTO articles (
                    title,
                    body,
                    url,
                    updated_at
                )
                VALUES ('', '', $1, datetime('now'))
                ON CONFLICT(url) DO UPDATE SET
                    is_deleted = 0,
                    updated_at = datetime('now')
                RETURNING id, title, body, created_at, url
                ",
            )
            .bind(&url)
            .fetch_one(pool)
            .await
            .with_context(|| format!("failed to add article with url {url}"))?;

            Ok(article)
        }
    }
}

#[tauri::command]
pub async fn get_article_count(db_instances: State<'_, DbInstances>) -> Result<i64, TauriError> {
    let instances = db_instances.0.read().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let (count,) =
                query_as::<_, (i64,)>("SELECT COUNT(*) FROM articles WHERE is_deleted = 0")
                    .fetch_one(pool)
                    .await
                    .context("failed to count articles")?;

            Ok(count)
        }
    }
}

#[tauri::command]
pub async fn refresh_article(
    id: i32,
    db_instances: State<'_, DbInstances>,
) -> Result<(), TauriError> {
    let instances = db_instances.0.write().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query(
                r"
                UPDATE articles
                SET
                    title = '',
                    body = '',
                    text_content = '',
                    updated_at = datetime('now')
                WHERE id = ?
                ",
            )
            .bind(id)
            .execute(pool)
            .await
            .with_context(|| format!("failed to refresh article with id {id}"))?;

            Ok(())
        }
    }
}

#[tauri::command]
pub async fn delete_article(
    id: i32,
    db_instances: State<'_, DbInstances>,
) -> Result<u64, TauriError> {
    let instances = db_instances.0.write().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance")?;

    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let result = query(
                r"
                UPDATE articles
                SET
                    is_deleted = 1,
                    title = '',
                    body = '',
                    text_content = '',
                    updated_at = datetime('now')
                WHERE id = ?
                  AND is_deleted = 0
                ",
            )
            .bind(id)
            .execute(pool)
            .await
            .with_context(|| format!("failed to delete article with id {id}"))?;

            Ok(result.rows_affected())
        }
    }
}

#[tauri::command]
pub async fn pick_import_file(
    app: tauri::AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), TauriError> {
    let urls: Vec<String> = crate::file_helpers::pick_and_read_json(&app)
        .context("failed to pick and read import file")?;

    for url in urls {
        add_article(url.clone(), db_instances.clone())
            .await
            .with_context(|| format!("failed to import article with url {url}"))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn pick_export_file(
    app: tauri::AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), TauriError> {
    let instances = db_instances.0.read().await;

    let db = instances
        .get(DB_URL)
        .context("failed to get database instance")?;

    let urls = match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => query_scalar::<_, String>(
            r"
                SELECT url
                FROM articles
                WHERE is_deleted = 0
                ORDER BY created_at
                ",
        )
        .fetch_all(pool)
        .await
        .context("failed to fetch article URLs for export")?,
    };

    crate::file_helpers::pick_and_write_json(&app, &urls, "read_later.json")
        .context("failed to write exported articles")?;

    Ok(())
}
