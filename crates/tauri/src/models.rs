use crate::schema::{articles, settings};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = articles)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = articles)]
pub struct NewArticle {
    pub title: String,
    pub body: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = settings)]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub default_value: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = settings)]
pub struct UpdateSetting {
    pub name: String,
    pub value: String,
}
