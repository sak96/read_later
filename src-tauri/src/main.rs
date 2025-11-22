#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    use std::sync::Mutex;
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let mut conn = article_manager_lib::db::establish_connection(&app_handle);
            article_manager_lib::db::run_migrations(&mut conn);

            app.manage(article_manager_lib::commands::AppState {
                app: Mutex::new(app_handle),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            article_manager_lib::commands::get_articles,
            article_manager_lib::commands::get_article,
            article_manager_lib::commands::add_article,
            article_manager_lib::commands::delete_article,
            article_manager_lib::commands::get_setting,
            article_manager_lib::commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
