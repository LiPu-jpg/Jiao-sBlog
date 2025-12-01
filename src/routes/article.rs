use rocket::{get, State};
use rocket_dyn_templates::{Template, context};
use crate::db;
use sqlx::PgPool;

#[get("/<id>")]
pub async fn article(id: i32, pool: &State<PgPool>) -> Template {
    // 获取文章
    let article = match db::get_article_by_id(id, &pool).await {
        Ok(article) => article,
        Err(_) => {
            return Template::render("error", context! {
                message: "文章不存在"
            });
        }
    };

    // 使用 Markdown 渲染文章内容
    let parser = pulldown_cmark::Parser::new(&article.content_md);
    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    // 传递给模板的上下文要和模板里变量名一致
    Template::render("article", context! {
        title: &article.title,
        article: &article,
        article_html: html_content,
    })
}
