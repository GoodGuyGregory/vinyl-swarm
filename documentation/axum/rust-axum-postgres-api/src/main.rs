use axum::{response::IntoResponse, routing::get, Json, Router};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Pool};
use std::env;
use std::error::Error;
use std::sync::Arc;


pub struct AppState {
    db: Pool<Postgres>,
}

/// basic status check endpoint
async fn status_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres, and Axum";

    let json_response = serde_json::json!({
        "status": "Success",
        "message": MESSAGE
    });

    // return the response
    Json(json_response)
}


async fn connect_to_database() -> Result<PgPool, Box<dyn Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

        
    println!("✅ Connection to the database is successful!");
    Ok(pool)
}

/// the main API logic 
#[tokio::main]
async fn main() {

    match connect_to_database().await {
        Ok(pool) => {
            let app_state = Arc::new(AppState { db: pool.clone() });
              // create the app
            let app = Router::new().route("/api/status", get(status_check_handler));
            
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