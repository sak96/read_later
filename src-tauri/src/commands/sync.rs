use crate::commands::settings::{get_setting, set_setting};
use crate::error::TauriError;
use crate::models::{ArticleSync, DB_URL};

use anyhow::{Context, Result};
use blake3;
use chrono::{NaiveDateTime, Utc};
use reqwest_dav::types::list_cmd::{ListEntity, ListFile};
use reqwest_dav::types::{Auth, Depth};
use reqwest_dav::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime, State, ipc::Channel};
use tauri_plugin_sql::DbInstances;

#[derive(Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub count_processed: usize,
    pub total_count: usize,
}

fn url_to_path(url: &str) -> String {
    let hash = blake3::hash(url.as_bytes());
    format!("{}.json", hash.to_hex())
}

fn setup_webdav_client(
    url: String,
    username: String,
    password: String,
    auth_type: &str,
) -> Result<Client> {
    let auth = match auth_type {
        "basic" => Auth::Basic(username, password),
        "digest" => Auth::Digest(username, password),
        _ => Auth::Anonymous,
    };

    ClientBuilder::new()
        .set_host(url)
        .set_auth(auth)
        .build()
        .context("failed to build WebDAV client")
}

fn iso_to_timestamp(iso_str: &str) -> i64 {
    NaiveDateTime::parse_from_str(iso_str, "%Y-%m-%d %H:%M:%S")
        .map_or(0, |naive| naive.and_utc().timestamp())
}

async fn get_remote_entities(
    client: &Client,
    sync_path: &str,
    last_synced_at: i64,
) -> Result<Vec<ListFile>> {
    if client.list(sync_path, Depth::Number(0)).await.is_err() {
        client
            .mkcol(sync_path)
            .await
            .context("failed to create WebDAV sync directory")?;
    }

    let entities = client
        .list(sync_path, Depth::Number(1))
        .await
        .context("failed to list WebDAV sync directory")?;

    Ok(entities
        .into_iter()
        .filter_map(|entity| {
            if let ListEntity::File(file) = entity
                && file.last_modified.timestamp() > last_synced_at
            {
                Some(file)
            } else {
                None
            }
        })
        .collect())
}

async fn get_local_sync_data(
    pool: &sqlx::SqlitePool,
    last_synced_at: i64,
) -> Result<Vec<ArticleSync>> {
    sqlx::query_as::<_, ArticleSync>(
        r"
        SELECT url, created_at, updated_at, is_deleted
        FROM articles
        WHERE datetime(updated_at) > datetime(?, 'unixepoch')
        ",
    )
    .bind(last_synced_at)
    .fetch_all(pool)
    .await
    .context("failed to fetch local articles for sync")
}

async fn reconcile_and_process(
    client: &Client,
    pool: &sqlx::SqlitePool,
    sync_path: &str,
    local_articles: Vec<ArticleSync>,
    remote_entities: Vec<ListFile>,
    progress_channel: Channel<SyncProgress>,
) -> Result<()> {
    use std::collections::HashSet;

    let mut all_hashes = HashSet::new();

    for article in &local_articles {
        all_hashes.insert(url_to_path(&article.url));
    }

    for entity in &remote_entities {
        if let Some(filename) = entity.href.split('/').next_back() {
            all_hashes.insert(filename.to_string());
        }
    }

    let total = all_hashes.len();

    for (i, hash) in all_hashes.iter().enumerate() {
        let path = format!("{sync_path}/{hash}");

        let local_article = local_articles
            .iter()
            .find(|article| &url_to_path(&article.url) == hash);

        let remote_article = match client.get(&path).await {
            Ok(response) => {
                let content = response
                    .text()
                    .await
                    .with_context(|| format!("failed to read remote article: {path}"))?;

                serde_json::from_str::<ArticleSync>(&content).ok()
            }
            Err(_) => None,
        };

        match (local_article, remote_article) {
            (Some(local), Some(remote)) => {
                let local_ts = iso_to_timestamp(&local.updated_at);
                let remote_ts = iso_to_timestamp(&remote.updated_at);

                if local_ts > remote_ts {
                    let content = serde_json::to_string(local)
                        .context("failed to serialize local article")?;

                    client
                        .put(&path, content)
                        .await
                        .with_context(|| format!("failed to upload local article: {path}"))?;
                } else if remote_ts > local_ts {
                    sqlx::query(
                        r"
                        UPDATE articles SET
                            updated_at = $1,
                            is_deleted = $2,
                            title = CASE WHEN $2 = 1 THEN '' ELSE title END,
                            body = CASE WHEN $2 = 1 THEN '' ELSE body END,
                            text_content = CASE WHEN $2 = 1 THEN '' ELSE text_content END
                        WHERE url = $3
                        ",
                    )
                    .bind(&remote.updated_at)
                    .bind(remote.is_deleted)
                    .bind(&remote.url)
                    .execute(pool)
                    .await
                    .context("failed to update article from remote")?;
                }
            }

            (Some(local), None) => {
                let content =
                    serde_json::to_string(local).context("failed to serialize local article")?;

                client
                    .put(&path, content)
                    .await
                    .with_context(|| format!("failed to upload local article: {path}"))?;
            }

            (None, Some(remote)) => {
                sqlx::query(
                    r"
                    INSERT INTO articles (
                        url,
                        created_at,
                        updated_at,
                        is_deleted,
                        title,
                        body,
                        text_content
                    )
                    VALUES ($1, $2, $3, $4, '', '', '')
                    ON CONFLICT(url) DO UPDATE SET
                        created_at = excluded.created_at,
                        updated_at = excluded.updated_at,
                        is_deleted = excluded.is_deleted,
                        title = CASE
                            WHEN excluded.is_deleted = 1 THEN ''
                            ELSE title
                        END,
                        body = CASE
                            WHEN excluded.is_deleted = 1 THEN ''
                            ELSE body
                        END,
                        text_content = CASE
                            WHEN excluded.is_deleted = 1 THEN ''
                            ELSE text_content
                        END
                    ",
                )
                .bind(&remote.url)
                .bind(&remote.created_at)
                .bind(&remote.updated_at)
                .bind(remote.is_deleted)
                .execute(pool)
                .await
                .context("failed to insert remote article")?;
            }

            (None, None) => {}
        }

        progress_channel
            .send(SyncProgress {
                count_processed: i + 1,
                total_count: total,
            })
            .context("failed to send sync progress")?;
    }

    Ok(())
}

#[tauri::command]
pub async fn sync_articles<R: Runtime>(
    _app_handle: AppHandle<R>,
    db_instances: State<'_, DbInstances>,
    progress_channel: Channel<SyncProgress>,
) -> Result<(), TauriError> {
    sync_articles_inner(db_instances, progress_channel)
        .await
        .map_err(TauriError::from)
}

async fn sync_articles_inner(
    db_instances: State<'_, DbInstances>,
    progress_channel: Channel<SyncProgress>,
) -> Result<()> {
    let webdav_enabled = get_setting("webdavEnabled".to_string(), db_instances.clone())
        .await
        .unwrap_or_else(|_| "false".to_string())
        == "true";

    if !webdav_enabled {
        let instances = db_instances.0.write().await;

        let tauri_plugin_sql::DbPool::Sqlite(pool) =
            instances.get(DB_URL).context("database is not loaded")?;

        sqlx::query("DELETE FROM articles WHERE is_deleted = 1")
            .execute(pool)
            .await
            .context("failed to delete locally deleted articles")?;

        return Ok(());
    }

    let url = get_setting("webdavUrl".to_string(), db_instances.clone())
        .await
        .context("failed to get WebDAV URL")?;

    let username = get_setting("webdavUsername".to_string(), db_instances.clone())
        .await
        .unwrap_or_default();

    let password = get_setting("webdavPassword".to_string(), db_instances.clone())
        .await
        .unwrap_or_default();

    let path = get_setting("webdavPath".to_string(), db_instances.clone())
        .await
        .unwrap_or_default();

    let auth_type = get_setting("webdavAuthType".to_string(), db_instances.clone())
        .await
        .unwrap_or_default();

    let client = setup_webdav_client(url, username, password, &auth_type)
        .context("failed to initialize WebDAV client")?;

    let sync_path = format!(
        "{}/.io.github.sak.read.it.later",
        path.trim_end_matches('/')
    );

    let instances = db_instances.0.read().await;

    let tauri_plugin_sql::DbPool::Sqlite(pool) =
        instances.get(DB_URL).context("database is not loaded")?;

    let new_synced_at = Utc::now().timestamp();

    let last_synced_at = get_setting("lastSyncedAt".to_string(), db_instances.clone())
        .await
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i64>()
        .context("invalid lastSyncedAt setting")?;

    let remote_entities = get_remote_entities(&client, &sync_path, last_synced_at)
        .await
        .context("failed to get remote sync data")?;

    let local_articles = get_local_sync_data(pool, last_synced_at)
        .await
        .context("failed to get local sync data")?;

    reconcile_and_process(
        &client,
        pool,
        &sync_path,
        local_articles,
        remote_entities,
        progress_channel,
    )
    .await
    .context("failed to reconcile articles")?;

    set_setting(
        "lastSyncedAt".to_string(),
        new_synced_at.to_string(),
        db_instances.clone(),
    )
    .await
    .context("failed to update lastSyncedAt")?;

    sqlx::query(
        r"
        DELETE FROM articles
        WHERE is_deleted = 1
        AND datetime(updated_at) < datetime(?, 'unixepoch')
        ",
    )
    .bind(last_synced_at)
    .execute(pool)
    .await
    .context("failed to clean up deleted articles")?;

    Ok(())
}
