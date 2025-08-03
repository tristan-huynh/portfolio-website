use rocket::Request;

use rocket_dyn_templates::{Template, handlebars, context};

use self::handlebars::{Handlebars, JsonRender};

#[get("/")]
pub fn index() -> Template {
    Template::render("pages/index", context! {
        title: "Portfolio",
        name: Some("Welcome"),
        items: vec!["Portfolio", "About", "Contact"],
    })
}

#[get("/hello/<name>")]
pub fn hello(name: &str) -> Template {
    Template::render("pages/index", context! {
        title: "Hello",
        name: Some(name),
        items: vec!["One", "Two", "Three"],
    })
}

#[get("/about")]
pub fn about() -> Template {
    Template::render("pages/about", context! {
        title: "About",
        parent: "layout",
    })
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    Template::render("error/404", context! {
        uri: req.uri()
    })
}

fn wow_helper(
    h: &handlebars::Helper<'_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        out.write("<b><i>")?;
        out.write(&param.value().render())?;
        out.write("</b></i>")?;
    }

    Ok(())
}

pub fn customize(hbs: &mut Handlebars) {
    hbs.register_helper("wow", Box::new(wow_helper));
    hbs.register_template_string("pages/about", r#"
        {{#*inline "page"}}

        <section id="about">
          <h1>About - Here's another page!</h1>
        </section>

        {{/inline}}
        {{> layout}}
    "#).expect("valid HBS template");
}
