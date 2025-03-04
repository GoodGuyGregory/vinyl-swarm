use axum::{response::IntoResponse, routing::get, Json, Router};

/// basic status check endpoint
async fn health_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres, and Axum";

    let json_response = serde_json::json!({
        "status": "Success",
        "message": MESSAGE
    });

    // return the response
    Json(json_response)
}


/// the main API logic 
#[tokio::main]
async fn main() {
    // create the app
    let app = Router::new().route("/api/healthchecker", get(health_check_handler));
    
    println!("🛸 Server started successfully");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}