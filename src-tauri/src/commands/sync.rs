use crate::commands::settings::{get_setting, set_setting};
use crate::models::{ArticleSync, DB_URL};
use blake3;
use chrono::{DateTime, Utc};
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
    auth_type: String,
) -> Client {
    let auth = match auth_type.as_str() {
        "basic" => Auth::Basic(username, password),
        "digest" => Auth::Digest(username, password),
        _ => Auth::Anonymous,
    };

    ClientBuilder::new()
        .set_host(url)
        .set_auth(auth)
        .build()
        .expect("Failed to build WebDAV client")
}

fn iso_to_timestamp(iso_str: &str) -> i64 {
    iso_str
        .parse::<DateTime<Utc>>()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

async fn get_remote_entities(
    client: &Client,
    sync_path: &str,
    last_synced_at: i64,
) -> Result<Vec<ListFile>, String> {
    let entities = client
        .list(sync_path, Depth::Number(1))
        .await
        .map_err(|e| e.to_string())?;

    Ok(entities
        .into_iter()
        .filter_map(|e| {
            if let ListEntity::File(file) = e
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
) -> Result<Vec<ArticleSync>, String> {
    sqlx::query_as::<_, ArticleSync>("SELECT url, created_at, updated_at, is_deleted FROM articles WHERE datetime(updated_at) > datetime(?, 'unixepoch')")
        .bind(last_synced_at)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn reconcile_and_process(
    client: &Client,
    pool: &sqlx::SqlitePool,
    sync_path: &str,
    local_articles: Vec<ArticleSync>,
    remote_entities: Vec<ListFile>,
    progress_channel: Channel<SyncProgress>,
) -> Result<(), String> {
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
        let path = format!("{}/{}", sync_path, hash);

        let local_article = local_articles.iter().find(|a| &url_to_path(&a.url) == hash);

        let remote_resp = client.get(&path).await;
        let remote_article = if let Ok(resp) = remote_resp {
            let content = resp.text().await.map_err(|e| e.to_string())?;
            serde_json::from_str::<ArticleSync>(&content).ok()
        } else {
            None
        };

        match (local_article, remote_article) {
            (Some(local), Some(remote)) => {
                let local_ts = iso_to_timestamp(&local.updated_at);
                let remote_ts = iso_to_timestamp(&remote.updated_at);

                if local_ts > remote_ts {
                    let content = serde_json::to_string(&local).map_err(|e| e.to_string())?;
                    client
                        .put(&path, content)
                        .await
                        .map_err(|e| e.to_string())?;
                } else if remote_ts > local_ts {
                    sqlx::query("UPDATE articles SET updated_at = ?, is_deleted = ? WHERE url = ?")
                        .bind(&remote.updated_at)
                        .bind(remote.is_deleted)
                        .bind(&remote.url)
                        .execute(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
            (Some(local), None) => {
                let content = serde_json::to_string(&local).map_err(|e| e.to_string())?;
                client
                    .put(&path, content)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            (None, Some(remote)) => {
                sqlx::query("INSERT INTO articles (url, created_at, updated_at, is_deleted) VALUES (?, ?, ?, ?)")
                    .bind(&remote.url)
                    .bind(&remote.created_at)
                    .bind(&remote.updated_at)
                    .bind(remote.is_deleted)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            (None, None) => {}
        }

        let _ = progress_channel.send(SyncProgress {
            count_processed: i + 1,
            total_count: total,
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn sync_articles<R: Runtime>(
    _app_handle: AppHandle<R>,
    db_instances: State<'_, DbInstances>,
    progress_channel: Channel<SyncProgress>,
) -> Result<(), String> {
    let webdav_enabled =
        get_setting("WEBDAV_ENABLED".to_string(), db_instances.clone()).await? == "true";
    if !webdav_enabled {
        return Ok(());
    }

    let url = get_setting("WEBDAV_URL".to_string(), db_instances.clone()).await?;
    let username = get_setting("WEBDAV_USERNAME".to_string(), db_instances.clone()).await?;
    let password = get_setting("WEBDAV_PASSWORD".to_string(), db_instances.clone()).await?;
    let path = get_setting("WEBDAV_PATH".to_string(), db_instances.clone()).await?;
    let auth_type = get_setting("WEBDAV_AUTH_TYPE".to_string(), db_instances.clone()).await?;

    // ... (rest of the setup code)
    let client = setup_webdav_client(url, username, password, auth_type);
    let sync_path = format!("{}/.io.github.sak.read.it.later", &path);

    let instances = db_instances.0.read().await;
    let tauri_plugin_sql::DbPool::Sqlite(pool) = instances.get(DB_URL).ok_or("db not loaded")?;

    let new_synced_at = Utc::now().timestamp();
    let last_synced_at = get_setting("LAST_SYNCED_AT".to_string(), db_instances.clone())
        .await
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i64>()
        .unwrap_or(0);

    let remote_entities = get_remote_entities(&client, &sync_path, last_synced_at).await?;
    let local_articles = get_local_sync_data(pool, last_synced_at).await?;

    reconcile_and_process(
        &client,
        pool,
        &sync_path,
        local_articles,
        remote_entities,
        progress_channel,
    )
    .await?;

    set_setting(
        "LAST_SYNCED_AT".to_string(),
        new_synced_at.to_string(),
        db_instances.clone(),
    )
    .await?;

    Ok(())
}
