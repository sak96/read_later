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
