use rocket_dyn_templates::Template;
use rocket::response::Redirect;
use rocket_dyn_templates::context;
use rocket::{get, post, form::Form, State};
use crate::db;
use crate::models::{UserLoginForm, NewArticleForm, NewTagForm, User as UserModel};
use sqlx::PgPool;
use rocket::request::{self, FromRequest, Request};
use rocket::http::{Status, Cookie, CookieJar};
use rocket::outcome::Outcome;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

//------------------------------------
// Session Store
//------------------------------------
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
        let pool = match req.rocket().state::<PgPool>() {
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
// 管理员登录页面
//------------------------------------
#[get("/login")]
pub fn login_page() -> Template {
    Template::render("admin/login", context! {})
}

//------------------------------------
// 管理员登录（返回 session_id cookie）
//------------------------------------
#[post("/login", data = "<form>")]
pub async fn admin_login(
    form: Form<UserLoginForm>,
    pool: &State<PgPool>,
    sessions: &State<SessionStore>,
    jar: &CookieJar<'_>,
) -> Result<Redirect, Status> {
    let login = form.into_inner();

    let user = sqlx::query_as::<_, UserModel>(
        r#"SELECT * FROM "user" WHERE username = $1 AND role = 'admin'"#
    )
    .bind(&login.username)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    let user = match user {
        Some(u) if u.password == login.password => u, // 生产环境需 hash+salt
        _ => return Err(Status::Unauthorized),
    };

    // 生成 session_id
    let session_id = uuid::Uuid::new_v4().to_string();
    sessions.lock().unwrap().insert(session_id.clone(), user.id);

    // 写入 cookie
    jar.add(Cookie::new("session_id", session_id));

    Ok(Redirect::to("/admin/dashboard"))
}

//------------------------------------
// 管理员登出
//------------------------------------
#[post("/logout")]
pub fn admin_logout(sessions: &State<SessionStore>, jar: &CookieJar<'_>) -> Redirect {
    if let Some(cookie) = jar.get("session_id") {
        sessions.lock().unwrap().remove(cookie.value());
        jar.remove(cookie.clone());
    }
    Redirect::to("/admin/login")
}

//------------------------------------
// Dashboard
//------------------------------------
#[get("/dashboard")]
pub async fn dashboard(_admin: AdminGuard) -> Template {
    Template::render("admin/dashboard", context! {})
}

//------------------------------------
// 文章管理
//------------------------------------
#[derive(Serialize)]
pub struct ArticleJson {
    id: i32,
    title: String,
    content_md: String,
    created_at: String,
}

#[get("/articles_data")]
pub async fn articles_data(_admin: AdminGuard, pool: &State<PgPool>) -> Json<Vec<ArticleJson>> {
    let articles = db::get_all_articles(pool.inner()).await.unwrap_or_default();
    
    let data: Vec<ArticleJson> = articles.into_iter().map(|a| ArticleJson {
        id: a.id,
        title: a.title,
        content_md: a.content_md,
        created_at: a.created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "未知时间".to_string()),
    }).collect();

    Json(data)
}

#[get("/articles")]
pub async fn articles_page(_admin: AdminGuard, pool: &State<PgPool>) -> Template {
    let articles = db::get_all_articles(pool.inner()).await.unwrap_or_default();
    Template::render("admin/articles", context! { articles })
}

#[get("/articles/new")]
pub async fn new_article_page(_admin: AdminGuard, pool: &State<PgPool>) -> Template {
    let tags = db::get_all_tags(pool.inner()).await.unwrap_or_default();
    Template::render("admin/new_article", context! { tags })
}

#[post("/articles", data = "<form>")]
pub async fn create_article(_admin: AdminGuard, form: Form<NewArticleForm>, pool: &State<PgPool>) -> Redirect {
    let NewArticleForm { title, content_md, tag_ids } = form.into_inner();
    let _ = db::create_article(pool.inner(), &title, &content_md, &tag_ids).await;
    Redirect::to("/admin/articles")
}

#[get("/articles/<id>/edit")]
pub async fn edit_article_page(_admin: AdminGuard, id: i32, pool: &State<PgPool>) -> Template {
    // 1. 获取文章及其标签
    let article: crate::models::Article = match db::get_article_by_id(id, pool.inner()).await {
        Ok(a) => a,
        Err(_) => return Template::render("error", context! { message: "文章不存在" }),
    };

    // 2. 包装成新的结构体传给模板（可选，因为现在 tags 已经是 Vec<i32>）
    #[derive(Serialize)]
    struct ArticleWithTagIds<'a> {
        id: i32,
        title: &'a str,
        content_md: &'a str,
        tag_ids: Vec<i32>,
    }

    let article_with_ids = ArticleWithTagIds {
        id: article.id,
        title: &article.title,
        content_md: &article.content_md,
        tag_ids: article.tags.clone(), // 🔥 直接 clone
    };

    // 3. 获取所有标签
    let tags = db::get_all_tags(pool.inner()).await.unwrap_or_default();

    Template::render("admin/edit_article", context! { article: article_with_ids, tags })
}

#[post("/articles/<id>", data = "<form>")]
pub async fn update_article(_admin: AdminGuard, id: i32, form: Form<NewArticleForm>, pool: &State<PgPool>) -> Redirect {
    let NewArticleForm { title, content_md, tag_ids } = form.into_inner();
    let _ = db::update_article(pool.inner(), id, &title, &content_md, &tag_ids).await;
    Redirect::to("/admin/articles")
}

#[post("/articles/<id>/delete")]
pub async fn delete_article(_admin: AdminGuard, id: i32, pool: &State<PgPool>) -> Redirect {
    let _ = db::delete_article(pool.inner(), id).await;
    Redirect::to("/admin/articles")
}

//------------------------------------
// 标签管理
//------------------------------------
#[get("/tags")]
pub async fn tags_page(_admin: AdminGuard, pool: &State<PgPool>) -> Template {
    let tags = db::get_all_tags(pool.inner()).await.unwrap_or_default();
    Template::render("admin/tags", context! { tags })
}

#[get("/tags/new")]
pub async fn new_tag_page(_admin: AdminGuard) -> Template {
    Template::render("admin/new_tag", context! {})
}

#[post("/tags", data = "<form>")]
pub async fn create_tag(
    _admin: AdminGuard,
    form: Form<NewTagForm>,
    pool: &State<PgPool>
) -> Template {
    let new_tag = form.into_inner();

    match db::create_tag(pool.inner(), &new_tag.name).await {
        Ok(_) => {
            // 创建成功，重新渲染标签列表
            let tags = db::get_all_tags(pool.inner()).await.unwrap_or_default();
            Template::render("admin/tags", context! { tags, message: "创建成功" })
        }
        Err(e) => {
            eprintln!("创建标签失败: {:?}", e);
            let tags = db::get_all_tags(pool.inner()).await.unwrap_or_default();
            Template::render("admin/tags", context! { tags, message: format!("创建失败: {}", e) })
        }
    }
}


#[post("/tags/<id>/delete")]
pub async fn delete_tag(_admin: AdminGuard, id: i32, pool: &State<PgPool>) -> Redirect {
    let _ = db::delete_tag(pool.inner(), id).await;
    Redirect::to("/admin/tags")
}
