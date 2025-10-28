use rocket::*;
use rocket_session::{Session};
use rocket_dyn_templates::Template;
use rocket::form::Form;
use sqlx::PgPool;
use crate::db::{create_article, get_all_articles, get_article_by_id, update_article, delete_article as db_delete_article};
use crate::models::{Article, Tag};

#[derive(FromForm)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

// 登录
#[post("/login", data = "<form>")]
pub fn login(form: Form<LoginForm>, mut session: Session<'_, ()>) -> Redirect {
    if form.username == "admin" && form.password == "123456" {
        session.insert("admin", true).unwrap();
        Redirect::to(uri!("/admin/articles"))
    } else {
        Redirect::to(uri!("/admin/login"))
    }
}

// 登出
#[get("/logout")]
pub fn logout(mut session: Session<'_, ()>) -> Redirect {
    session.remove("admin");
    Redirect::to(uri!("/admin/login"))
}

// 获取文章列表
#[get("/articles")]
pub async fn list_articles(session: Session<'_, ()>, pool: &State<PgPool>) -> Template {
    if !session.get::<bool>("admin").unwrap_or(false) {
        return Template::render("admin/login", ());
    }
    
    let articles = get_all_articles(pool).await.unwrap();
    Template::render("admin/articles", &articles)
}

// 新增文章页面
#[get("/new_article")]
pub fn new_article_page(session: Session<'_, ()>) -> Template {
    if !session.get::<bool>("admin").unwrap_or(false) {
        return Template::render("admin/login", ());
    }

    Template::render("admin/new_article", ())
}

// 创建新文章
#[post("/new_article", data = "<form>")]
pub async fn new_article(form: Form<ArticleForm>, session: Session<'_, ()>, pool: &State<PgPool>) -> Redirect {
    if !session.get::<bool>("admin").unwrap_or(false) {
        return Redirect::to(uri!("/admin/login"));
    }

    create_article(pool, form.title.as_str(), form.content_md.as_str(), &form.tag_ids).await.unwrap();
    Redirect::to(uri!("/admin/articles"))
}

// 修改文章页面
#[get("/edit_article/<id>")]
pub async fn edit_article_page(id: i32, session: Session<'_, ()>, pool: &State<PgPool>) -> Template {
    if !session.get::<bool>("admin").unwrap_or(false) {
        return Template::render("admin/login", ());
    }

    let article = get_article_by_id(id, pool).await.unwrap();
    Template::render("admin/edit_article", &article)
}

// 编辑文章
#[post("/edit_article/<id>", data = "<form>")]
pub async fn edit_article(id: i32, form: Form<ArticleForm>, session: Session<'_, ()>, pool: &State<PgPool>) -> Redirect {
    if !session.get::<bool>("admin").unwrap_or(false) {
        return Redirect::to(uri!("/admin/login"));
    }

    update_article(pool, id, form.title.as_str(), form.content_md.as_str(), &form.tag_ids).await.unwrap();
    Redirect::to(uri!("/admin/articles"))
}

// 删除文章
#[post("/delete_article/<id>")]
pub async fn delete_article(id: i32, session: Session<'_, ()>, pool: &State<PgPool>) -> Redirect {
    if !session.get::<bool>("admin").unwrap_or(false) {
        return Redirect::to(uri!("/admin/login"));
    }

    db_delete_article(id, pool).await.unwrap();
    Redirect::to(uri!("/admin/articles"))
}
