use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct ArticleId {
    pub id: i32,
}
#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct ArticleEntry {
    pub id: i32,
    pub title: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub default_value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum IntentEvent {
    TextIntent(String),
    Empty,
}
