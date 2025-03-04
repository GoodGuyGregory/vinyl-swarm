use axum::{response::IntoResponse, routing::get, Json, Router};
use tokio::net::TcpListener;
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Pool};
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::sync::Arc;

pub struct AppState {
    db: Pool<Postgres>,
}

pub async fn status_handler() -> impl IntoResponse {
    let message_status: &str = "vinyl swarm running: 👽 ";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": message_status
    });

    Json(json_response)
}

async fn connect_to_database() -> Result<PgPool, Box<dyn Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("ERROR: DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ Connection to the database is successful!");
    Ok(pool)
}



// Basics of using AXUM with mySQL
#[tokio::main]
async fn main() {
    
    match connect_to_database().await {
        Ok(pool) => {
            let app_state = Arc::new(AppState { db: pool.clone() });
              // create the app
            let app = Router::new().route("/api/status", get(status_handler));
            
            println!("🛸 Server started successfully");

            let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

            axum::serve(listener, app).await.unwrap();
        }
        // catch the Error
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
        }
    }
}