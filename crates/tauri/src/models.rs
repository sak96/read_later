use tauri_plugin_sql::{Migration, MigrationKind};

pub const DB_URL: &str = "sqlite:article_manager.db";

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct ArticleId {
    pub id: i32,
}
#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct ArticleEntry {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub default_value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum IntentEvent {
    TextIntent(String),
    Empty,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum FetchProgress {
    Downloading(String),
    Parsing(String),
}

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/2025-11-22-000000_create_initial_tables.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_article_url",
            sql: include_str!("../migrations/2025-11-29-000000_add_article_url.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
