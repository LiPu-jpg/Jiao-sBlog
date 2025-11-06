#[macro_use] extern crate rocket;

mod db;
mod routes;
mod models;
mod auth;

use dotenvy::dotenv;
use rocket::{Rocket, Build};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use routes::*;
use crate::auth::jwt::JWTConfig;

#[launch]
async fn rocket() -> Rocket<Build> {
    dotenv().ok();

    // 创建数据库连接池
    let database_url = "postgres://bloguser:060628@localhost:5432/blog";
    let pool = db::create_pool(database_url).await.expect("Failed to create database pool");
    
    // 创建 JWT 配置
    let jwt_config = JWTConfig::new("your-secret-key-here-change-in-production");

    rocket::build()
        .attach(Template::fairing())
        .manage(pool)
        .manage(jwt_config)
        .mount("/", routes![index::index])
        .mount("/about", routes![about::about])
        .mount("/friends", routes![friends::friends])
        .mount("/travel", routes![travel::travel])
        .mount("/article", routes![article::article])
        .mount("/tags", routes![tags::tags, tags::tag_articles])
        .mount("/static", FileServer::from("static"))
        .mount("/archive", routes![archive::archive])
        // Admin 路由
        .mount("/admin", routes![
            admin::login_page,
            admin::admin_login,
            admin::dashboard,
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
            admin::articles_data,
        ])
}