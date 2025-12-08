use sqlx::FromRow;
use chrono::NaiveDateTime;
use serde::Serialize;
use rocket::form::FromForm;

/// 文章模型
#[derive(FromRow, Debug, Serialize, Clone)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content_md: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub tags: Vec<i32>,
}

/// 标签模型
#[derive(FromRow, Debug, Serialize, Clone)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

/// 登录用户
#[derive(FromRow, Debug, Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
    pub created_at: Option<NaiveDateTime>,
}

/// 登录表单
#[derive(FromForm, Debug)]
pub struct UserLoginForm {
    pub username: String,
    pub password: String,
}

/// 新文章表单
#[derive(FromForm, Debug)]
pub struct NewArticleForm {
    pub title: String,
    pub content_md: String,
    pub tag_ids: Vec<i32>,
}

/// 新标签表单
#[derive(FromForm, Debug)]
pub struct NewTagForm {
    pub name: String,
}


/// 首页视图结构
#[derive(FromRow, Debug, Serialize)]
pub struct RecentArticleView {
    pub id: i32,
    pub title: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(FromRow, Debug, Serialize)]
pub struct TagWithCount {
    pub id: i32,
    pub name: String,
    pub article_count: Option<i64>,
}


#[derive(Debug, Serialize)]
pub struct BlogStats {
    pub article_count: i64,
    pub tag_count: i64,
    pub days_running: i64,
    pub visit_count: i64,
}


#[derive(Debug, sqlx::FromRow)]
pub struct SiteInfo {
    pub created_at: chrono::NaiveDateTime,
    pub visit_count: i64,
}

/*
#[derive(Debug, Serialize)]
pub struct ArticleWithTags {
    pub id: i32,
    pub title: String,
    pub content_md: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub tags: Vec<Tag>,
}

// 归档条目视图模型
#[derive(Debug, Serialize)]
pub struct ArchiveEntry {
    pub year: i32,
    pub month: u32,
    pub count: i64,
    pub articles: Vec<Article>,
}

// 文章-标签关联模型
#[derive(FromRow, Debug)]
pub struct ArticleTag {
    pub article_id: i32,
    pub tag_id: i32,
} */