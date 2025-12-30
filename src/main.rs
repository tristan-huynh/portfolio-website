use axum::{Form, Router, extract::State, http::StatusCode, response::Html, routing::get};
use lettre::message::{Mailbox, Message, header};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use serde::Deserialize;
use std::env;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

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
        .route("/contact", get(contact).post(contact_post))
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

// deal with contact me stuff

#[derive(Deserialize)]
struct ContactForm {
    name: String,
    email: String,
    message: String,
}

async fn contact_post(State(_tera): State<Arc<Tera>>, Form(form): Form<ContactForm>) -> StatusCode {
    match send_email(&form.name, &form.email, &form.message).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Email send failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn send_email(name: &str, email: &str, message: &str) -> Result<(), String> {
    let smtp_username =
        env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set".to_string())?;
    let smtp_password =
        env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set".to_string())?;
    let smtp_server = env::var("SMTP_SERVER").map_err(|_| "SMTP_SERVER not set".to_string())?;
    let smtp_server_port: u16 = env::var("SMTP_SERVER_PORT")
        .map_err(|_| "SMTP_SERVER_PORT not set".to_string())?
        .parse()
        .map_err(|_| "SMTP_SERVER_PORT is not a valid number".to_string())?;

    let message_email = Message::builder()
        .from(Mailbox::new(None, smtp_username.parse().unwrap()))
        .reply_to(Mailbox::new(Some(name.to_string()), email.parse().map_err(|_| "Invalid email address".to_string())?))
        .to(Mailbox::new(None, smtp_username.parse().unwrap()))
        .subject("New Contact Form Submission")
        .header(header::ContentType::TEXT_PLAIN)
        .body(format!(
            "Name: {}\nEmail: {}\n\nMessage:\n{}",
            name, email, message
        ))
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(smtp_username.clone(), smtp_password);
    let mailer = SmtpTransport::relay(smtp_server.as_str())
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .port(smtp_server_port)
        .credentials(creds)
        .build();
    mailer
        .send(&message_email)
        .map_err(|e| format!("Failed to send email: {}", e))?;
    Ok(())
}
