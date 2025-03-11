use std::sync::Arc;
use axum::{
    response::IntoResponse, 
    routing::{get},
    Json, 
    Router
};


// internal modules 
use crate::{
    AppState,
    routes::records::{
        // wishlists:
        get_users_wishlist, add_to_user_wishlist, put_wishlist_record, remove_wishlist_record, remove_user_wishlist,
        list_all_records, find_record, create_new_record, edit_record, delete_record_by_id
    },
    routes::record_stores::{
        // user_record_stores:
        get_user_record_stores, add_existing_record_store, add_user_record_store, delete_user_record_store,
        list_all_stores, create_record_store, edit_record_store, find_record_store, delete_record_store},
    routes::users::{
        list_all_users, find_specific_user, create_user, edit_user, delete_user, 
        get_user_records, create_user_record, put_user_record, remove_user_record, remove_all_user_records},
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
        .route("/records", get(list_all_records)
                                            .post(create_new_record))
        .route("/records/{id}", get(find_record)
                                                    .patch(edit_record)
                                                    .delete(delete_record_by_id))
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
                        .delete(delete_user))
        .route("/users/records/{user_id}", get(get_user_records)
                                                            .put(put_user_record)
                                                            .post(create_user_record)
                                                            .patch(remove_user_record)
                                                                .delete(remove_all_user_records))
        .route("/records/wishlist/{user_id}", get(get_users_wishlist)
                                                    .post(add_to_user_wishlist)
                                                .put(put_wishlist_record)
                                            .delete(remove_user_wishlist )
                                        .patch(remove_wishlist_record))
        .route("/record_stores/{user_id}", get(get_user_record_stores)
                                                            .post(add_user_record_store)
                                                            .put(add_existing_record_store)
                                                            .delete(delete_user_record_store));

    // return the router
    Router::new()
        .nest("/api", api_routes)
        .with_state(app_state)

}