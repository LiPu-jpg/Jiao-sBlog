use crate::db;
use rocket::{get, State};
use rocket_dyn_templates::{Template, context};
use sqlx::PgPool;

#[get("/")]
pub async fn tags(pool: &State<PgPool>) -> Template {
    let tags = db::get_all_tags(&pool).await.unwrap();
    // ✅ 用冒号
    Template::render("tags", context! {
        tags: tags
    })
}

#[get("/<tag_id>")]
pub async fn tag_articles(tag_id: i32, pool: &State<PgPool>) -> Template {
    let articles = db::get_articles_by_tag(&pool, tag_id).await.unwrap();
    // ✅ 用冒号
    Template::render("tag_articles", context! {
        articles: articles
    })
}
