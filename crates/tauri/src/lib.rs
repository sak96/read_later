pub mod commands;
pub mod models;
pub mod parse;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_sharetarget::init())
        .plugin(tauri_plugin_tts::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(models::DB_URL, models::get_migrations())
                .build(),
        )
        .setup(|_app| Ok(()))
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
