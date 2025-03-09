use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    Json,
    response::IntoResponse,
};


// internal modules 
use crate::{
    AppState,
    routes::records::list_all_records,
    routes::record_stores::list_all_stores,
};

pub async fn status_handler() -> impl IntoResponse {
    let message_status: &str = "vinyl swarm running: 👽 ";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": message_status
    });

    Json(json_response)
}

// api documentation:
// https://docs.rs/crate/axum/latest/source/src/docs/routing/nest.md
pub fn create_router(app_state: Arc<AppState>) -> Router {
    // create the router for all CRUD endpoints
    let api_routes = Router::new()
    // status route
        .route("/status", get(status_handler))
        .route("/records", get(list_all_records))
        .route("/stores", get(list_all_stores));

    // return the router
    Router::new()
        .nest("/api", api_routes)
        .with_state(app_state)

}