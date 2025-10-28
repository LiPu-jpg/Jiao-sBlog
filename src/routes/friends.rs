use rocket_dyn_templates::{Template, context};

#[get("/")]
pub fn friends() -> Template {
    Template::render("friends", context! { title: "友链" })
}
