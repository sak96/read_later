pub mod commands;
pub mod db;
pub mod models;
pub mod schema;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::sync::Mutex;
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let mut conn = crate::db::establish_connection(&app_handle);
            crate::db::run_migrations(&mut conn);

            app.manage(crate::commands::AppState {
                app: Mutex::new(app_handle),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::get_articles,
            crate::commands::get_article,
            crate::commands::add_article,
            crate::commands::delete_article,
            crate::commands::get_setting,
            crate::commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
