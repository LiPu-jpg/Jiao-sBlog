use rocket_dyn_templates::Template;
use rocket::response::Redirect;
use rocket_session::Session;
use rocket_dyn_templates::context;
use rocket::{get, post, form::Form, State};
use crate::db;
use crate::models::{Article, Tag};
use sqlx::PgPool;

#[derive(Default)]
struct SessionData {
    admin: Option<bool>,
}

#[derive(FromForm)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[get("/login")]
pub fn login_page() -> Template {
    // 渲染登录页面
    Template::render("admin/login", ())
}

#[post("/login", data = "<form>")]
pub async fn login(form: Form<LoginForm>, mut session: Session<'_, SessionData>) -> Redirect {
    let login_data = form.into_inner();

    if login_data.username == "Jiao" && login_data.password == "060628" {
        session.tap(|data| {
            data.admin = Some(true);
        });

        Redirect::to("/admin/dashboard")
    } else {
        Redirect::to("/login") 
    }
}

#[get("/logout")]
pub fn logout(mut session: Session<'_, SessionData>) -> Redirect {
    session.tap(|data| {
        data.admin = None;
    });

    Redirect::to("/login")
}

// 获取文章列表
#[get("/articles")]
pub async fn articles_page(pool: &State<PgPool>) -> Template {
    let articles = match db::get_all_articles(&pool).await {
        Ok(articles) => articles,
        Err(_) => vec![],
    };
    Template::render("admin/articles", context! { articles })
}

// 渲染文章创建页面
#[get("/articles/new")]
pub async fn new_article_page(pool: &State<PgPool>) -> Template {
    let tags = match db::get_all_tags(&pool).await {
        Ok(tags) => tags,
        Err(_) => vec![],
    };
    Template::render("admin/new_article", context! { tags })
}

// 处理文章创建请求
#[post("/articles", data = "<form>")]
pub async fn create_article(form: Form<NewArticleForm>, pool: &State<PgPool>) -> Redirect {
    let NewArticleForm { title, content_md, tag_ids } = form.into_inner();
    let tag_ids: Vec<i32> = tag_ids.iter().map(|id| *id).collect();

    if let Err(_) = db::create_article(&pool, &title, &content_md, &tag_ids).await {
        // 错误处理
        return Redirect::to("/admin/articles");
    }

    Redirect::to("/admin/articles")
}

// 渲染编辑页面
#[get("/articles/<id>/edit")]
pub async fn edit_article_page(id: i32, pool: &State<PgPool>) -> Template {
    let article = match db::get_article_by_id(id, &pool).await {
        Ok(article) => article,
        Err(_) => {
            return Template::render("error", context! { message: "文章不存在" });
        }
    };
    let tags = match db::get_all_tags(&pool).await {
        Ok(tags) => tags,
        Err(_) => vec![],
    };
    Template::render("admin/edit_article", context! {
        article,
        tags
    })
}

// 处理文章更新请求
#[post("/articles/<id>", data = "<form>")]
pub async fn update_article(id: i32, form: Form<NewArticleForm>, pool: &State<PgPool>) -> Redirect {
    let NewArticleForm { title, content_md, tag_ids } = form.into_inner();
    let tag_ids: Vec<i32> = tag_ids.iter().map(|id| *id).collect();

    if let Err(_) = db::update_article(&pool, id, &title, &content_md, &tag_ids).await {
        return Redirect::to("/admin/articles");
    }

    Redirect::to("/admin/articles")
}

// 删除文章
#[post("/articles/<id>/delete")]
pub async fn delete_article(id: i32, pool: &State<PgPool>) -> Redirect {
    if let Err(_) = db::delete_article(&pool, id).await {
        return Redirect::to("/admin/articles");
    }

    Redirect::to("/admin/articles")
}

// 用于处理创建和更新文章的表单结构
#[derive(FromForm)]
pub struct NewArticleForm {
    pub title: String,
    pub content_md: String,
    pub tag_ids: Vec<i32>,  // 多个标签ID
}

// 显示标签管理页面
#[get("/tags")]
pub async fn tags_page(pool: &State<PgPool>) -> Template {
    let tags = match db::get_all_tags(&pool).await {
        Ok(tags) => tags,
        Err(_) => vec![],
    };
    Template::render("admin/tags", context! { tags })
}

// 渲染创建标签页面
#[get("/tags/new")]
pub fn new_tag_page() -> Template {
    Template::render("admin/new_tag", ())
}

// 处理标签创建请求
#[post("/tags", data = "<form>")]
pub async fn create_tag(form: Form<NewTagForm>, pool: &State<PgPool>) -> Redirect {
    let new_tag = form.into_inner();

    if let Err(_) = db::create_tag(&pool, &new_tag.name).await {
        // 错误处理：如果创建失败，重定向到标签页面
        return Redirect::to("/admin/tags");
    }

    Redirect::to("/admin/tags")  // 标签创建成功后，返回标签管理页面
}

// 删除标签
#[post("/tags/<id>/delete")]
pub async fn delete_tag(id: i32, pool: &State<PgPool>) -> Redirect {
    if let Err(_) = db::delete_tag(&pool, id).await {
        return Redirect::to("/admin/tags");
    }

    Redirect::to("/admin/tags")
}


// 用于处理创建标签的表单结构
#[derive(FromForm)]
pub struct NewTagForm {
    pub name: String,  // 标签名称
}