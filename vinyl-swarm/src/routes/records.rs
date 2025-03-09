use std::sync::Arc;

use axum::{
    body::Body,
    routing::get,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Router,
};

use chrono::offset;
use serde_json::json;

use crate::AppState;
use crate::models::record::{RecordModel, FilterOptions};

/// GET all records from the database
pub async fn list_all_records(
    Query(opts): Query<FilterOptions>,
    State(data): State<Arc<AppState>>,
) -> impl IntoResponse {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // query as the record model and return all the records
    let query_result = sqlx::query_as!(
        RecordModel,
        "SELECT * FROM records ORDER BY artist LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    ).fetch_all(&data.db)
    .await;

    match query_result {
        // no error
        Ok(records) => {
            // found records return them to client
            let json_response = serde_json::json!({
                "status": "success",
                "results": records.len(),
                "records": records,
            });
            (StatusCode::OK, Json(json_response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Something bad happened while fetching all records",
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }

}