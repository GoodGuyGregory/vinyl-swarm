use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

// internal modules 
use crate::{
    handler::{
        create_note_handler, delete_note_handler, edit_note_handler, note_list_handler, read_note_handler, status_check_handler
    },
    AppState,
};



pub fn create_router(app_state: Arc<AppState>) -> Router {
    // create the router for the CRUD endpoints
    Router::new()
        .route("/api/status", get(status_check_handler))
        .route("/api/notes/", post(create_note_handler))
        .route("/api/notes", get(note_list_handler))
        .route(
            "/api/notes/{id}",
            get(read_note_handler)
                .patch(edit_note_handler)
                .delete(delete_note_handler)
        )
        .with_state(app_state)
    
}