use sqlx::FromRow;
use chrono::NaiveDateTime;
use serde::Serialize;
use rocket::form::FromForm;

// 文章模型
#[derive(FromRow, Debug, Serialize, Clone)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content_md: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub tags: Vec<i32>, 
}

// 标签模型
#[derive(FromRow, Debug, Serialize, Clone)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}



// 用户模型 - 用于认证
#[derive(FromRow, Debug, Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
    pub created_at: Option<NaiveDateTime>,
}

// 用户登录表单
#[derive(FromForm, Debug)]
pub struct UserLoginForm {
    pub username: String,
    pub password: String,
}

// 新建文章表单
#[derive(FromForm, Debug)]
pub struct NewArticleForm {
    pub title: String,
    pub content_md: String,
    pub tag_ids: Vec<i32>,
}

// 新建标签表单
#[derive(FromForm, Debug)]
pub struct NewTagForm {
    pub name: String,
}
/*
// 文章详情视图模型（包含标签信息）
#[derive(Debug, Serialize)]
pub struct ArticleWithTags {
    pub id: i32,
    pub title: String,
    pub content_md: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub tags: Vec<Tag>,
}

// 标签文章计数视图模型
#[derive(FromRow, Debug, Serialize)]
pub struct TagWithCount {
    pub id: i32,
    pub name: String,
    pub article_count: i64,
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
}
*/