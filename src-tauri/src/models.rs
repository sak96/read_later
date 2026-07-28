use tauri_plugin_sql::{Migration, MigrationKind};

pub const DB_URL: &str = "sqlite:article_manager.db";

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, sqlx::FromRow)]
pub struct ArticleId {
    pub id: i32,
}
#[derive(Serialize, Deserialize, PartialEq, Clone, sqlx::FromRow)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Snippet {
    pub prefix: String,
    pub match_text: Option<String>,
    pub suffix: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ArticleEntry {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub snippet: Snippet,
    pub created_at: String,
}
#[derive(Serialize, Deserialize, PartialEq, Clone, sqlx::FromRow)]
pub struct ArticleSync {
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_deleted: i32,
}

#[derive(sqlx::FromRow)]
pub struct ArticleEntryRow {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub text_content: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, sqlx::FromRow)]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub default_value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, sqlx::FromRow)]
pub struct PronunciationRule {
    pub match_pattern: String,
    pub replacement: String,
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
        Migration {
            version: 3,
            description: "add_text_content",
            sql: include_str!("../migrations/2026-04-03-000000_add_text_content.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "add_article_sync",
            sql: include_str!("../migrations/2026-05-23-000000_add_article_sync.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "add_unique_constraint_to_articles",
            sql: include_str!(
                "../migrations/2026-05-24-144135_add_unique_constraint_to_articles.sql"
            ),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "add_pronunciation_rules",
            sql: include_str!("../migrations/2026-07-28-000000_add_pronunciation_rules.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
