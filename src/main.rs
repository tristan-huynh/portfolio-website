use axum::http::{HeaderValue, header};
use axum::{Form, Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use lettre::message::{Mailbox, Message, header as lettre_header};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::ConnectInfo;
use std::net::SocketAddr;

struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<u64>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
        }
    }

    fn check_limit(&self, ip: &str, max_requests: usize, window_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut requests = self.requests.lock().unwrap();
        let ip_requests = requests.entry(ip.to_string()).or_default();
        
        // Remove old requests outside window
        ip_requests.retain(|&timestamp| now - timestamp < window_secs);
        
        if ip_requests.len() >= max_requests {
            return false;
        }
        
        ip_requests.push(now);
        true
    }
}

struct AppState {
    tera: Arc<Tera>,
    rate_limiter: Arc<RateLimiter>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // local dev vs production
    let templates_pattern = if Path::new("src/templates").exists() {
        "src/templates/**/*.html"
    } else {
        "templates/**/*.html"
    };

    // local dev vs production
    let static_path = if Path::new("src/static").exists() {
        "src/static"
    } else {
        "static"
    };

    let tera = match Tera::new(templates_pattern) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
let app_state = Arc::new(AppState {
    tera: Arc::new(tera),
    rate_limiter: Arc::new(RateLimiter::new()),
});

    let app = Router::new()
        .route("/", get(home))
        .route("/projects", get(projects))
        .route("/contact", get(contact).post(contact_post))
        .nest_service("/static", ServeDir::new(static_path))
        .with_state(app_state.clone())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; \
                script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://challenges.cloudflare.com; \
                style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; \
                img-src 'self' data:; \
                font-src 'self' https://cdn.jsdelivr.net; \
                connect-src 'self' https://challenges.cloudflare.com; \
                frame-src https://challenges.cloudflare.com;"
            ),
        ));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listenting on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn home(State(app_state): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("current_page", "main");
    context.insert("page_title", "Main");
    context.insert("page_description", "Welcome to my portfolio website.");
    let html = app_state.tera
        .render("home.html", &context)
        .expect("Failed to render template");
    Html(html)
}

async fn projects(State(app_state): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("current_page", "projects");
    context.insert("page_title", "Projects");
    context.insert("page_description", "A showcase of my projects.");
    let html = app_state.tera
        .render("projects.html", &context)
        .expect("Failed to render template");
    Html(html)
}

async fn contact(State(app_state): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("current_page", "contact");
    context.insert("page_title", "Contact");
    context.insert("page_description", "Get in touch with me.");
    let html = app_state.tera
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
    #[serde(rename = "cf-turnstile-response")]
    turnstile_response: String,
}

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

async fn contact_post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Form(form): Form<ContactForm>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let ip = addr.ip().to_string();
    if !app_state.rate_limiter.check_limit(&ip, 3, 60) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                message: "Too many requests. Please try again in a minute.".to_string(),
            }),
        ));
    }
    
    if form.name.len() > 100 || form.email.len() > 100 || form.message.len() > 1000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                message: "Input fields exceed maximum allowed length".to_string(),
            }),
        ));
    }
    match verify_turnstile(&form.turnstile_response).await {
        Ok(_) => match send_email(&form.name, &form.email, &form.message).await {
            Ok(_) => Ok(StatusCode::OK),
            Err(e) => {
                eprintln!("Email send failed: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        message: "Failed to send email".to_string(),
                    }),
                ))
            }
        },
        Err(e) => {
            eprintln!("Turnstile verification failed: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    message: "Verification failed. Please try again.".to_string(),
                }),
            ))
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
        .reply_to(Mailbox::new(
            Some(name.to_string()),
            email
                .parse()
                .map_err(|_| "Invalid email address".to_string())?,
        ))
        .to(Mailbox::new(None, smtp_username.parse().unwrap()))
        .subject("New Contact Form Submission")
        .header(lettre_header::ContentType::TEXT_PLAIN)
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

// cloudflare turnstile serversided work
async fn verify_turnstile(token: &str) -> Result<bool, String> {
    let secret =
        env::var("TURNSTILE_SECRET_KEY").map_err(|_| "TURNSTILE_SECRET_KEY not set".to_string())?;

    let params = serde_json::json!({
        "secret": secret,
        "response": token,
    });

    let response = reqwest::Client::new()
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("Turnstile verification request failed: {}", e))?;

    let result: TurnstileResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Turnstile response: {}", e))?;

    if !result.success {
        if let Some(codes) = result.error_codes {
            return Err(format!("Turnstile verification failed: {:?}", codes));
        }
        return Err("Turnstile verification failed".to_string());
    }

    Ok(true)
}
