use rocket::{get, State};
use rocket_dyn_templates::{Template, context};
use crate::db;
use sqlx::PgPool;
use std::collections::HashMap;

#[get("/")]
pub async fn archive(pool: &State<PgPool>) -> Template {
    // 从数据库中获取按年份分组的文章
    let articles_grouped_by_year = db::get_articles_grouped_by_year(pool.inner())
        .await
        .unwrap_or_default();

    // 转换成 HashMap<String, Vec<Article>>，方便 Tera 迭代 year, articles
    let mut map = HashMap::new();
    for (year, articles) in articles_grouped_by_year {
        map.insert(year.to_string(), articles);
    }

    // ✅ 注意这里用冒号，不是 =>
    Template::render("archive", context! {
        articles_grouped_by_year: map
    })
}
