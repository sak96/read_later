use crate::db::establish_connection;
use crate::models::*;
use crate::schema::*;
use diesel::prelude::*;
use readability_rust::Readability;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub app: Mutex<tauri::AppHandle>,
}

#[tauri::command]
pub fn get_articles(state: State<AppState>) -> Result<Vec<Article>, String> {
    let app = state.app.lock().unwrap();
    let mut conn = establish_connection(&app);

    articles::table
        .select(Article::as_select())
        .order(articles::created_at.desc())
        .load(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_article(id: i32, state: State<AppState>) -> Result<Article, String> {
    let app = state.app.lock().unwrap();
    let mut conn = establish_connection(&app);

    articles::table
        .find(id)
        .select(Article::as_select())
        .first(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_article(url: String, state: State<'_, AppState>) -> Result<Article, String> {
    let html = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let mut parser =
        Readability::new(&html, None).map_err(|e| format!("Failed to parse: {:?}", e))?;

    let article_data = parser.parse().ok_or("Failed to extract article")?;

    let app = state.app.lock().unwrap();
    let mut conn = establish_connection(&app);

    let new_article = NewArticle {
        title: article_data.title.unwrap_or_else(|| "Untitled".to_string()),
        body: article_data.content.unwrap_or_default(),
    };

    diesel::insert_into(articles::table)
        .values(&new_article)
        .returning(Article::as_returning())
        .get_result(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn render_readable_content(html: String) -> Result<String, String> {
    Ok(html)
}

#[tauri::command]
pub fn get_setting(name: String, state: State<AppState>) -> Result<String, String> {
    let app = state.app.lock().unwrap();
    let mut conn = establish_connection(&app);

    settings::table
        .find(&name)
        .select(settings::value)
        .first(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(name: String, value: String, state: State<AppState>) -> Result<(), String> {
    let app = state.app.lock().unwrap();
    let mut conn = establish_connection(&app);

    let update = UpdateSetting {
        name: name.clone(),
        value,
    };

    diesel::update(settings::table.find(&name))
        .set(&update)
        .execute(&mut conn)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
