use std::sync::Arc;
use axum::{
    response::IntoResponse, 
    routing::{get, patch}, 
    Json, 
    Router
};


// internal modules 
use crate::{
    AppState,
    routes::records::list_all_records,
    routes::record_stores::{list_all_stores, create_record_store, edit_record_store, find_record_store, delete_record_store},
    routes::users::{list_all_users, find_specific_user, create_user, edit_user, delete_user},
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
        .route("/stores", get(list_all_stores)
                                            .post(create_record_store))
        .route("/stores/{id}",    
                            get(find_record_store)
                                        .patch(edit_record_store)
                                        .delete(delete_record_store))
        .route("/users", 
        get(list_all_users)
                    .post(create_user))
        .route("/users/{id}", get(find_specific_user)
                            .patch(edit_user)
                        .delete(delete_user));

    // return the router
    Router::new()
        .nest("/api", api_routes)
        .with_state(app_state)

}