use rocket::{get, State};
use rocket_dyn_templates::{Template, context};
use sqlx::PgPool;
use crate::db;
use crate::models::BlogStats;

#[get("/")]
pub async fn index(pool: &State<PgPool>) -> Template {
    db::increment_visit(&pool).await.ok();

    let recent_articles = db::get_recent_articles(pool, 5)
        .await
        .unwrap_or_default();

    let popular_tags = db::get_popular_tags(pool, 10)
        .await
        .unwrap_or_default();

    let stats = db::get_blog_stats(pool)
        .await
        .unwrap_or_else(|_| BlogStats {
            article_count: 0,
            tag_count: 0,
            days_running: 0,
            visit_count: 0,
        });

    Template::render("index", context! {
        title: "首页",
        recent_articles,
        popular_tags,

        article_count: stats.article_count,
        tag_count: stats.tag_count,
        running_days: stats.days_running,
        visit_count: stats.visit_count,
    })
}
