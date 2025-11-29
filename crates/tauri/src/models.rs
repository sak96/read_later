use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri_plugin_sql::{Migration, MigrationKind};

pub const DB_URL: &str = "sqlite:article_manager.db";

pub fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_initial_tables",
        sql: include_str!("../migrations/2025-11-22-000000_create_initial_tables.sql"),
        kind: MigrationKind::Up,
    }]
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Article {
    pub id: i32, // usually i64 in SQLite/SQLx, but i32 works if fits
    pub title: String,
    pub body: String,
    pub created_at: String,
}

// We don't strictly need a separate struct for Insert if we pass args directly,
// but it's fine to keep if you use it for frontend payloads.
#[derive(Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub default_value: String,
}
