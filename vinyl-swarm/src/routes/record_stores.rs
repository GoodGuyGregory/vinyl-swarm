use std::sync::Arc;

use axum::{
    body, extract::{Query, State}, http::StatusCode, response::IntoResponse, Json
};
use chrono::offset;
use serde_json::json;
use crate::AppState;
use crate::models::store::{RecordStoreModel, FilterOptions, CreateRecordStoreSchema};


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

/// create_record_store
/// this handler is used for adding additional record stores that are worth shopping at.
/// POST methods are recommended for this handler
pub async fn create_record_store(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecordStoreSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    // create the insert statement to add another record store
    let query_result = sqlx::query_as!(
        RecordStoreModel, 
        "INSERT INTO record_stores (store_name, store_address, store_city, store_state, store_zip, phone_number, website)
        VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        body.store_name.to_string(),
        body.store_address.to_string(),
        body.store_city.to_string(),
        body.store_state.to_string(),
        body.store_zip.to_string(),
        body.phone_number.to_owned().unwrap_or("".to_string()),
        body.website.to_owned().unwrap_or("".to_string())
    ).fetch_one(&data.db)
    .await;  

    match query_result {
        Ok(created_record_store) => {
            let record_store_response = json!({
                "status": "success",
                "record_store": created_record_store
            });

            return Ok((StatusCode::CREATED, Json(record_store_response)));
        }
        Err(e) => {
            // otherwise it's not a duplicate and something went wrong
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": format!("{:?}", e)}))
            ));
        }
    }

}