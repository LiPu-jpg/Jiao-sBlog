#[macro_use] extern crate rocket;

mod db;
mod routes;
mod utils;
mod models;

use dotenv::dotenv;
use rocket::{Rocket, Build};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use routes::*;


#[launch]
async fn rocket() -> Rocket<Build> {
    // 加载 .env 文件中的环境变量
    dotenv().ok();

    // 创建数据库连接池
    let database_url = "postgres://bloguser:060628@localhost:5432/blog"; // 替换为你的数据库 URL
    let pool = db::create_pool(database_url).await.unwrap();

    rocket::build()
        .attach(Template::fairing()) // 启用模板渲染
        .manage(pool) // 管理数据库连接池
        .mount("/", routes![index::index]) // 挂载主页路由
        .mount("/about", routes![about::about]) // 挂载关于页路由
        .mount("/friends", routes![friends::friends]) // 挂载友链路由
        .mount("/travel", routes![travel::travel]) // 挂载旅行路由
        .mount("/article", routes![article::article]) // 挂载文章路由
        .mount("/tags", routes![tags::tags, tags::tag_articles]) // 挂载标签路由
        .mount("/static", FileServer::from("static")) // 提供静态文件
        .mount("/archive", routes![archive::archive]) // 挂载归档路由
        .mount("/admin", routes![admin::login_page]) // 挂载归档路由
        .mount("/admin", routes![
            admin::login_page, 
            admin::login, 
            admin::logout,
            admin::articles_page, 
            admin::new_article_page,
            admin::create_article, 
            admin::edit_article_page, 
            admin::update_article,
            admin::delete_article, 
            admin::tags_page,
            admin::new_tag_page, 
            admin::create_tag, 
            admin::delete_tag,
        ]) // 挂载管理员后台路由
}
