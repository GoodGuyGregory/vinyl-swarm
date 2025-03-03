use axum::{response::IntoResponse, routing::get, Json, Router};
use tokio::net::TcpListener;

pub async fn status_handler() -> impl IntoResponse {
    let message_status: &str = "Vinyl Swarm Running";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": message_status
    });

    Json(json_response)
}


#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/status", get(status_handler) );

    println!("Server started successfully at localhost:8080");

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}


