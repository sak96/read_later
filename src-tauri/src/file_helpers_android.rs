use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tauri::AppHandle;
use tauri_plugin_android_fs::AndroidFsExt;

pub fn pick_and_read_json<T: DeserializeOwned>(app: &AppHandle) -> Result<T, String> {
    let api = app.android_fs();
    if let Ok(Some(file_path)) = api
        .file_picker()
        .pick_file(None, &["application/json"], true)
    {
        let file: File = api
            .open_file_readable(&file_path)
            .map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| format!("Failed to parse file: {e}"))
    } else {
        Err("No file selected".into())
    }
}

pub fn pick_and_write_json<T: Serialize>(
    app: &AppHandle,
    data: &T,
    filename: &str,
) -> Result<(), String> {
    let api = app.android_fs();
    if let Ok(Some(file_path)) =
        api.file_picker()
            .save_file(None, filename, Some("application/json"), true)
    {
        let file: File = api
            .open_file_writable(&file_path)
            .map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, data).map_err(|e| format!("Failed to write file: {e}"))
    } else {
        Err("No save location selected".into())
    }
}
