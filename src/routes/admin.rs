use rocket_dyn_templates::Template;
use rocket::response::Redirect;
use rocket_dyn_templates::context;
use rocket::{get, post, form::Form, State};
use crate::db;
use crate::models::{UserLoginForm, NewArticleForm, NewTagForm, User as UserModel};
use sqlx::PgPool;
use crate::auth::jwt::{create_token, validate_token, JWTConfig};
use rocket::request::{self, FromRequest, Request};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::serde::json::Json;
use rocket::serde::Serialize;

//------------------------------------
// AdminGuard: JWT + 数据库验证管理员
//------------------------------------
pub struct AdminGuard(pub UserModel);

#[derive(Serialize)]
struct ArticleJson {
    id: i32,
    title: String,
    content_md: String,
    created_at: String,
}

#[get("/articles_data")]
pub async fn articles_data(_admin: AdminGuard, pool: &State<PgPool>) -> Json<Vec<ArticleJson>> {
    let articles = db::get_all_articles(pool.inner()).await.unwrap_or_default();
    
    // 转换成前端可序列化结构，并处理 Option<NaiveDateTime>
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminGuard {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let jwt_config = match req.rocket().state::<JWTConfig>() {
            Some(config) => config,
            None => {
                eprintln!("JWTConfig not configured");
                return Outcome::Error((Status::InternalServerError, "JWT config not configured"));
            }
        };
        
        let pool = match req.rocket().state::<PgPool>() {
            Some(pool) => pool,
            None => {
                eprintln!("PgPool not configured");
                return Outcome::Error((Status::InternalServerError, "Database pool not configured"));
            }
        };

        let token = match req.headers().get_one("Authorization").and_then(|h| h.strip_prefix("Bearer ")) {
            Some(t) => t,
            None => return Outcome::Error((Status::Unauthorized, "Missing token")),
        };

        let claims = match validate_token(token, jwt_config) {
            Ok(c) => c,
            Err(_) => return Outcome::Error((Status::Unauthorized, "Invalid token")),
        };

        let user = match sqlx::query_as::<_, UserModel>(
            r#"SELECT * FROM "user" WHERE id = $1 AND role = 'admin'"#
        )
        .bind(claims.sub)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(u)) => u,
            Ok(None) => return Outcome::Error((Status::Unauthorized, "Not an admin")),
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Outcome::Error((Status::InternalServerError, "Database error"));
            }
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
// 管理员登录（返回 JWT）
//------------------------------------
#[post("/login", data = "<form>")]
pub async fn admin_login(
    form: Form<UserLoginForm>,
    pool: &State<PgPool>,
    jwt_config: &State<JWTConfig>,
) -> Result<String, Status> {
    let login = form.into_inner();

    let user = sqlx::query_as::<_, UserModel>(r#"SELECT * FROM "user" WHERE username = $1 AND role = 'admin'"#)
        .bind(&login.username)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            Status::InternalServerError
        })?;

    let user = match user {
        Some(u) if u.password == login.password => u, // 生产环境建议 hash + salt
        _ => return Err(Status::Unauthorized),
    };

    let token = create_token(user.id, jwt_config.inner());
    Ok(token)
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
    let article = match db::get_article_by_id(id, pool.inner()).await {
        Ok(a) => a,
        Err(_) => return Template::render("error", context! { message: "文章不存在" }),
    };
    let tags = db::get_all_tags(pool.inner()).await.unwrap_or_default();
    Template::render("admin/edit_article", context! { article, tags })
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
pub async fn create_tag(_admin: AdminGuard, form: Form<NewTagForm>, pool: &State<PgPool>) -> Redirect {
    let new_tag = form.into_inner();
    let _ = db::create_tag(pool.inner(), &new_tag.name).await;
    Redirect::to("/admin/tags")
}

#[post("/tags/<id>/delete")]
pub async fn delete_tag(_admin: AdminGuard, id: i32, pool: &State<PgPool>) -> Redirect {
    let _ = db::delete_tag(pool.inner(), id).await;
    Redirect::to("/admin/tags")
}