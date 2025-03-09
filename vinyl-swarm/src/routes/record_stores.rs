use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::offset;
use crate::AppState;
use crate::models::store::{RecordStoreModel, FilterOptions};


/// GET all record stores from the database
/// returns all record_stores 
/// params include the FilterOptions Struct to allow for pagination,
/// this will return 10 if there is no chosen option query parameter.
pub async fn list_all_stores(
    Query(opts): Query<FilterOptions>,
    State(data): State<Arc<AppState>>,
) -> impl IntoResponse {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // query as the record model and return all the records
    let query_result = sqlx::query_as!(
        RecordStoreModel,
        "SELECT * FROM record_stores ORDER BY store_name LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    ).fetch_all(&data.db)
    .await;

    match query_result {
        // no error
        Ok(record_stores) => {
            // found records return them to client
            let json_response = serde_json::json!({
                "status": "success",
                "results": record_stores.len(),
                "record_stores": record_stores,
            });
            (StatusCode::OK, Json(json_response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Something bad happened while fetching all record stores",
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}