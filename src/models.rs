use sqlx::FromRow;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(FromRow, Debug, Serialize)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content_md: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(FromRow, Debug, Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(FromRow, Debug)]
pub struct ArticleTag {
    pub article_id: i32,
    pub tag_id: i32,
}
