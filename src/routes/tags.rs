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
    // 1️⃣ 获取当前标签信息
    let tag = match db::get_tag_by_id(pool, tag_id).await {
        Ok(tag) => tag,
        Err(_) => return Template::render("error", context! { message: "标签不存在" }),
    };

    // 2️⃣ 获取该标签下的所有文章
    let mut articles = match db::get_articles_by_tag(pool, tag_id).await {
        Ok(list) => list,
        Err(_) => vec![],
    };

    // 3️⃣ 为每篇文章填充完整的 tags 列表
    for article in articles.iter_mut() {
        article.tags = match db::get_tags_by_article_id(pool, article.id).await {
            Ok(tags) => tags.into_iter().map(|t| t.id).collect(),
            Err(_) => vec![],
};

    }

    // 4️⃣ 渲染模板
    Template::render("tag_articles", context! {
        tag: tag,       // 当前标签
        articles: articles
    })
}
