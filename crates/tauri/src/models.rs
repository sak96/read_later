use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Article {
    pub id: i32, // usually i64 in SQLite/SQLx, but i32 works if fits
    pub title: String,
    pub body: String,
    pub created_at: String,
}

// We don't strictly need a separate struct for Insert if we pass args directly,
// but it's fine to keep if you use it for frontend payloads.
#[derive(Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub default_value: String,
}
