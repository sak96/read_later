use anyhow::{Context, Error, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

pub fn pick_and_read_json<T: DeserializeOwned>(app: &AppHandle) -> Result<T, Error> {
    let Some(file_path) = app.dialog().file().blocking_pick_file() else {
        return Err(Error::msg("No file selected"));
    };
    let path = file_path.as_path().context("could not get a path")?;
    let file = File::open(path).with_context(|| format!("Failed to open file {path:?}"))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).with_context(|| format!("Failed to parse file: {path:?}"))
}

pub fn pick_and_write_json<T: Serialize>(
    app: &AppHandle,
    data: &T,
    filename: &str,
) -> Result<(), Error> {
    let Some(file_path) = app
        .dialog()
        .file()
        .add_filter("JSON Files", &["json"])
        .set_file_name(filename)
        .blocking_save_file()
    else {
        return Err(Error::msg("No save location selected"));
    };
    let path = file_path.as_path().context("could not get a path")?;
    let file = File::create(path).with_context(|| format!("Could not create file: {path:?}"))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, data).with_context(|| format!("Failed to write file: {path:?}"))
}
