#[macro_use] extern crate rocket;

mod hbs;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![hbs::index])
        .mount("/templates", routes![hbs::hello, hbs::about])
        .mount("/static", FileServer::from(relative!("src/static")))
        .register("/", catchers![hbs::not_found])
        .attach(Template::custom(|engines| {
            hbs::customize(&mut engines.handlebars);
        }))
}
