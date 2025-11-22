use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use tauri::AppHandle;
use tauri::Manager;

pub fn establish_connection(app: &AppHandle) -> SqliteConnection {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

    let db_path = app_dir.join("article_manager.db");
    let database_url = db_path.to_str().expect("Failed to convert path to string");

    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn run_migrations(conn: &mut SqliteConnection) {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}
