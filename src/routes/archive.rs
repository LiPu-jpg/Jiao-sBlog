use rocket::{get, State};
use rocket_dyn_templates::Template;
use crate::db; // 引入 db.rs 中的函数
use sqlx::PgPool;
use rocket_dyn_templates::{context}; // 添加 context 的导入


#[get("/")]
pub async fn archive(pool: &State<PgPool>) -> Template {
    // 从数据库中获取按年份分组的文章
    let articles_grouped_by_year = db::get_articles_grouped_by_year(pool.inner()).await.unwrap();

    // 渲染页面
    Template::render("archive", context! {
        articles_grouped_by_year: articles_grouped_by_year
    })
}
