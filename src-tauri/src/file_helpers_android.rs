use anyhow::{Context, Error, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tauri::AppHandle;
use tauri_plugin_android_fs::AndroidFsExt;

pub fn pick_and_read_json<T: DeserializeOwned>(app: &AppHandle) -> Result<T, Error> {
    let api = app.android_fs();

    let Some(file_path) = api
        .picker()
        .pick_file(None, &["application/json"], true)
        .map_err(|e| anyhow::anyhow!(e))
        .context("failed to open file picker")?
    else {
        return Err(anyhow::anyhow!("No file selected"));
    };

    let file: File = api
        .open_file_readable(&file_path)
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| format!("Failed to open file: {file_path:?}"))?;

    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON file: {file_path:?}"))
}

pub fn pick_and_write_json<T: Serialize>(
    app: &AppHandle,
    data: &T,
    filename: &str,
) -> Result<(), Error> {
    let api = app.android_fs();

    let Some(file_path) = api
        .picker()
        .save_file(None, filename, Some("application/json"), true)
        .map_err(|e| anyhow::anyhow!(e))
        .context("failed to open save file picker")?
    else {
        return Err(anyhow::anyhow!("No save location selected"));
    };

    let file: File = api
        .open_file_writable(&file_path)
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| format!("Could not create file: {file_path:?}"))?;

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, data)
        .with_context(|| format!("Failed to write JSON file: {file_path:?}"))
}
