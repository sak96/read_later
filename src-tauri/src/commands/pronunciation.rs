use crate::models::*;
use regex::Regex;
use sqlx::{query, query_as, SqlitePool};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_sql::DbInstances;

async fn get_all_rules(pool: &SqlitePool) -> Result<Vec<PronunciationRule>, String> {
    query_as::<_, PronunciationRule>(
        "SELECT match_pattern, replacement, is_regex FROM pronunciation_rules ORDER BY match_pattern",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}

async fn save_rule(
    pool: &SqlitePool,
    match_pattern: &str,
    replacement: &str,
    is_regex: bool,
) -> Result<(), String> {
    if is_regex {
        Regex::new(match_pattern).map_err(|e| {
            eprintln!("regex failure: pattern={match_pattern} error={e}");
            format!("Invalid regex: {e}")
        })?;
    }
    query(
        r#"
        INSERT INTO pronunciation_rules (match_pattern, replacement, is_regex)
        VALUES ($1, $2, $3)
        ON CONFLICT(match_pattern) DO UPDATE SET
            replacement = $2,
            is_regex = $3
        "#,
    )
    .bind(match_pattern)
    .bind(replacement)
    .bind(is_regex)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_pronunciation_rules(
    db_instances: State<'_, DbInstances>,
) -> Result<Vec<PronunciationRule>, String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => get_all_rules(pool).await,
    }
}

#[tauri::command]
pub async fn save_pronunciation_rule(
    match_pattern: String,
    replacement: String,
    is_regex: bool,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            save_rule(pool, &match_pattern, &replacement, is_regex).await
        }
    }
}

#[tauri::command]
pub async fn delete_pronunciation_rule(
    match_pattern: String,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            query("DELETE FROM pronunciation_rules WHERE match_pattern = $1")
                .bind(&match_pattern)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn pick_pronunciation_import_file(
    app: AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let rules: Vec<PronunciationRule> = crate::file_helpers::pick_and_read_json(&app)?;
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => {
            let mut failures = 0;
            for rule in &rules {
                if save_rule(pool, &rule.match_pattern, &rule.replacement, rule.is_regex)
                    .await
                    .is_err()
                {
                    failures += 1;
                }
            }
            if failures == 0 {
                Ok(())
            } else {
                Err(format!("{failures} rules failed to save"))
            }
        }
    }
}

#[tauri::command]
pub async fn pick_pronunciation_export_file(
    app: AppHandle,
    db_instances: State<'_, DbInstances>,
) -> Result<(), String> {
    let instances = db_instances.0.read().await;
    let db = instances.get(DB_URL).ok_or("db not loaded")?;
    let rules = match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => get_all_rules(pool).await?,
    };
    crate::file_helpers::pick_and_write_json(&app, &rules, "read_later_rules.json")
}

pub async fn apply_pronunciation_rules(
    app: &AppHandle,
    paragraphs: Vec<String>,
) -> Result<Vec<String>, String> {
    let rules = {
        let instances = app.state::<tauri_plugin_sql::DbInstances>();
        let instances = instances.0.read().await;
        let db = instances.get(DB_URL).ok_or("db not loaded")?;
        match db {
            tauri_plugin_sql::DbPool::Sqlite(pool) => get_all_rules(pool).await?,
        }
    };

    if rules.is_empty() {
        return Ok(paragraphs);
    }

    let compiled_regex: Vec<(Regex, &String)> = rules
        .iter()
        .filter(|r| r.is_regex)
        .filter_map(|r| Regex::new(&r.match_pattern).ok().map(|re| (re, &r.replacement)))
        .collect();

    let plain: Vec<(&str, &str)> = rules
        .iter()
        .filter(|r| !r.is_regex)
        .map(|r| (r.match_pattern.as_str(), r.replacement.as_str()))
        .collect();

    Ok(paragraphs
        .into_iter()
        .map(|text| {
            let mut result = text;
            for (pattern, replacement) in &plain {
                result = result.replace(pattern, replacement);
            }
            for (re, replacement) in &compiled_regex {
                result = re.replace_all(&result, replacement.as_str()).to_string();
            }
            result
        })
        .collect())
}
