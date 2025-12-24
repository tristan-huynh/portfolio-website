use axum::{Router, extract::Path, response::Html, routing::get};
// use std::fs;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(home));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listenting on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> Html<&'static str> {
    Html("<h1>Hello world</h1>")
}
