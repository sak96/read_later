use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

pub fn pick_and_read_json<T: DeserializeOwned>(app: &AppHandle) -> Result<T, String> {
    let Some(file_path) = app.dialog().file().blocking_pick_file() else {
        return Err("No file selected".into());
    };
    let path = file_path.as_path().ok_or("could not get a path")?;
    let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(|e| format!("Failed to parse file: {e}"))
}

pub fn pick_and_write_json<T: Serialize>(
    app: &AppHandle,
    data: &T,
    filename: &str,
) -> Result<(), String> {
    let Some(file_path) = app
        .dialog()
        .file()
        .add_filter("JSON Files", &["json"])
        .set_file_name(filename)
        .blocking_save_file()
    else {
        return Err("No save location selected".into());
    };
    let path = file_path.as_path().ok_or("could not get a path")?;
    let file = File::create(path).map_err(|e| format!("Could not create file: {e}"))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, data).map_err(|e| format!("Failed to write file: {e}"))
}
