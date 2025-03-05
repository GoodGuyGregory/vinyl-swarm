
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Pool};
use std::env;
use std::error::Error;
use std::sync::Arc;

use route::create_router;

// add the modules
mod handler;
mod model;
mod schema;
mod route;



pub struct AppState {
    db: Pool<Postgres>,
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
            // init the app state
            let app_state = Arc::new(AppState { db: pool.clone() });
              // create the app
            
            // enable cors and the router for our app.
            let app = create_router(app_state);
            
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