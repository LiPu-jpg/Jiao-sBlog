use crate::db;
use rocket::{get, State};
use rocket_dyn_templates::Template;
use sqlx::PgPool;

#[get("/")]
pub async fn tags(pool: &State<PgPool>) -> Template {
    // 获取所有标签
    let tags = db::get_all_tags(&pool).await.unwrap();
    Template::render("tags", &tags)
}

#[get("/<tag_id>")]
pub async fn tag_articles(tag_id: i32, pool: &State<PgPool>) -> Template {
    // 根据标签 ID 获取所有相关文章
    let articles = db::get_articles_by_tag(&pool, tag_id).await.unwrap();
    Template::render("tag_articles", &articles)
}
