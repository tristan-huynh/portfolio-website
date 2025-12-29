use axum::{Router, extract::State, response::Html, routing::get};
// use std::fs;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let tera = match Tera::new("src/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let app = Router::new()
        .route("/", get(home))
        .route("/projects", get(projects))
        .route("/contact", get(contact))
        .nest_service("/static", ServeDir::new("src/static"))
        .with_state(Arc::new(tera));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listenting on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn home(State(tera): State<Arc<Tera>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("current_page", "main");
    context.insert("page_title", "Main");
    context.insert("page_description", "Welcome to my portfolio website.");
    let html = tera
        .render("home.html", &context)
        .expect("Failed to render template");
    Html(html)
}

async fn projects(State(tera): State<Arc<Tera>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("current_page", "projects");
    context.insert("page_title", "Projects");
    context.insert("page_description", "A showcase of my projects.");
    let html = tera
        .render("projects.html", &context)
        .expect("Failed to render template");
    Html(html)
}

async fn contact(State(tera): State<Arc<Tera>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("current_page", "contact");
    context.insert("page_title", "Contact");
    context.insert("page_description", "Get in touch with me.");
    let html = tera
        .render("contact.html", &context)
        .expect("Failed to render template");
    Html(html)
}
