use crate::models::*;
use crate::parse::{build_snippet, process_html};
use readabilityrs::Readability;
use sqlx::{query, query_as};
use std::sync::mpsc;
use tauri::{AppHandle, Listener, Manager, Runtime};
use tauri::{State, ipc::Channel};
use tauri_plugin_http::reqwest;
use tauri_plugin_sql::DbInstances;

const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

const HTML_FETCHER: &str = "html-fetcher";

pub fn fetch_rendered_html<R: Runtime>(app: &AppHandle<R>, url: &str) -> Result<String, String> {
    let (tx, rx) = mpsc::channel::<String>();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let webview = app
        .get_webview_window("main")
        .ok_or("Failed to get main webview")?;

    // Remember where the user was before opening external page.
    let previous_url = webview
        .url()
        .map(|u| u.to_string())
        .unwrap_or_else(|_| "http://localhost/home".to_string());

    let listener_id = app.listen(HTML_FETCHER, move |event| {
        let payload_str = event.payload().to_string();
        let parsed: Result<String, _> = serde_json::from_str(&payload_str);

        match parsed {
            Ok(json) => {
                let _ = tx.send(json.to_string());
            }
            Err(err) => {
                eprintln!("Failed to parse event payload: {err}");
            }
        }
        running_clone.store(false, Ordering::Relaxed);
    });

    let webview = app
        .get_webview_window("main")
        .ok_or("Failed to get main webview")?;

    // Navigate to the requested page.
    webview
        .eval(format!(r#"window.location.href = "{}";"#, url))
        .map_err(|e| e.to_string())?;

    // Keep attempting to inject the toolbar while capture mode is active.
    let injector_webview = webview.clone();
    let injector_running = running.clone();

    thread::spawn(move || {
        while injector_running.load(Ordering::Relaxed) {
            let _ = injector_webview.eval(
                r##"
                const bar = document.createElement("div");
                bar.id = "__tauri_capture_toolbar";

                Object.assign(bar.style, {
                    position: "fixed",
                    right: "20px",
                    bottom: "20px",
                    zIndex: "2147483647",
                    display: "flex",
                    flexDirection: "column",
                });

                const buttonStyle = {
                    margin: "0",
                    backgroundColor: "#0172ad",
                    color: "#eff1f4",
                    minWidth: "1.5em",
                    minHeight: "1.5em",
                    fontSize: "1.5em",
                    padding: "0.75rem 1.25rem",
                    border: "1px solid transparent",
                    borderRadius: "0.5rem",
                    cursor: "pointer",
                    fontWeight: "600",
                    textAlign: "center",
                    transition: "background-color .2s, border-color .2s, color .2s"
                };


                // OK button
                const ok = document.createElement("button");
                ok.textContent = "✓";
                ok.className = "contrast";
                Object.assign(ok.style, buttonStyle);

                ok.onclick = () => {
                    window.__TAURI__.event.emit(
                        "html-fetcher",
                        document.documentElement ? document.documentElement.outerHTML : ""
                    );
                };

                // Cancel button
                const cancel = document.createElement("button");
                cancel.textContent = "×";
                cancel.className = "secondary";
                Object.assign(cancel.style, buttonStyle);

                cancel.onclick = () => {
                    window.__TAURI__.event.emit("html-fetcher", "");
                };

                bar.appendChild(ok);
                bar.appendChild(cancel);

                if (document.body) {
                    document.body.appendChild(bar);
                }
                "##,
            );

            thread::sleep(Duration::from_millis(500));
        }
    });

    // Wait until the user presses OK.
    let html = rx.recv().map_err(|e| e.to_string())?;

    running.store(false, Ordering::Relaxed);

    app.unlisten(listener_id);
    // Return to previous page.
    let restore_js = format!(
        r#"window.location.href = "{}";"#,
        previous_url.replace('"', "\\\"")
    );

    let _ = webview.eval(&restore_js);

    Ok(html)
}

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
                on_progress
                    .send(FetchProgress::Downloading(article.url.to_string()))
                    .map_err(|e| e.to_string())?;

                let html = match sqlx::query_as::<_, (String,)>(
                    "SELECT value FROM settings WHERE name = 'iframe_fetcher'",
                )
                .fetch_one(pool)
                .await
                .ok()
                .map(|r| r.0)
                {
                    Some(v) if v == "true" => fetch_rendered_html(&app, &article.url)?,
                    _ => {
                        let client = reqwest::Client::new();
                        client
                            .get(&article.url)
                            .header(reqwest::header::USER_AGENT, CHROME_USER_AGENT)
                            .send()
                            .await
                            .map_err(|e| e.to_string())?
                            .text()
                            .await
                            .map_err(|e| e.to_string())?
                    }
                };

                let options = readabilityrs::ReadabilityOptions::builder()
                    .remove_title_from_content(true)
                    .clean_whitespace(false)
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
