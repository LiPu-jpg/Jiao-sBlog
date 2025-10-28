use rocket_dyn_templates::{Template, context};

#[get("/")]
pub fn about() -> Template {
    Template::render("about", context! { title: "关于" })
}
