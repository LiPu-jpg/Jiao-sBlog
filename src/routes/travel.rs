use rocket_dyn_templates::{Template, context};

#[get("/")]
pub fn travel() -> Template {
    Template::render("travel", context! { title: "足迹" })
}
