#[macro_use] extern crate rocket;

mod routes;
mod handlers;
mod models;
mod config;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            routes::pages::index,
            routes::pages::about,
            routes::pages::portfolio,
            routes::pages::contact,
        ])
        .mount("/api", routes![
            routes::api::contact_form,
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}