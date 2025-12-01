#[macro_use] 
extern crate rocket;

mod db;
mod routes;
mod models;

use dotenvy::dotenv;
use rocket::{Rocket, Build};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use rocket::http::{Status};
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::Outcome;


use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use routes::*;
use models::{User as UserModel};

pub type SessionStore = Arc<Mutex<HashMap<String, i32>>>; // session_id -> user_id

//------------------------------------
// AdminGuard: Cookie/Session + 数据库验证管理员
//------------------------------------
#[allow(dead_code)]
pub struct AdminGuard(pub UserModel);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminGuard {
    type Error = Status;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // 获取数据库连接池
        let pool = match req.rocket().state::<sqlx::PgPool>() {
            Some(p) => p,
            None => return Outcome::Error((Status::InternalServerError, Status::InternalServerError)),
        };

        // 获取 session 存储
        let sessions = match req.rocket().state::<SessionStore>() {
            Some(s) => s,
            None => return Outcome::Error((Status::InternalServerError, Status::InternalServerError)),
        };

        // 获取 cookie
        let jar = req.cookies();
        let session_id = match jar.get("session_id") {
            Some(c) => c.value().to_string(),
            None => return Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
        };

        // 查找 session 对应的 user_id
        let user_id = {
            let sessions = sessions.lock().unwrap();
            match sessions.get(&session_id) {
                Some(id) => *id,
                None => return Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
            }
        };

        // 查询管理员用户
        let user = match sqlx::query_as::<_, UserModel>(
            r#"SELECT * FROM "user" WHERE id = $1 AND role = 'admin'"#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(u)) => u,
            _ => return Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
        };

        Outcome::Success(AdminGuard(user))
    }
}

//------------------------------------
// Rocket launch
//------------------------------------
#[launch]
async fn rocket() -> Rocket<Build> {
    dotenv().ok();

    // 创建数据库连接池
    let database_url = "postgres://bloguser:060628@localhost:5432/blog";
    let pool = db::create_pool(database_url)
        .await
        .expect("Failed to create database pool");

    // 创建 session 存储
    let sessions: SessionStore = Arc::new(Mutex::new(HashMap::new()));

    rocket::build()
        .attach(Template::fairing())
        .manage(pool)
        .manage(sessions)
        // 前台路由
        .mount("/", routes![index::index])
        .mount("/about", routes![about::about])
        .mount("/friends", routes![friends::friends])
        .mount("/travel", routes![travel::travel])
        .mount("/article", routes![article::article])
        .mount("/tags", routes![tags::tags, tags::tag_articles])
        .mount("/archive", routes![archive::archive])
        .mount("/static", FileServer::from("static"))
        // 后台 Admin 路由
        .mount("/admin", routes![
            admin::login_page,
            admin::admin_login,
            admin::admin_logout,
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
