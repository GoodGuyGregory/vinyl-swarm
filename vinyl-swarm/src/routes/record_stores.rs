use std::sync::Arc;

use axum::{
    extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json
};

use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::models::store::{RecordStoreModel, FilterOptions, UpdateRecordStoreSchema, CreateRecordStoreSchema};


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

pub async fn find_record_store(
    Path(id): Path<Uuid>, 
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    // get the record store assuming it's a valid uuid
    let query_result = sqlx::query_as!(
        RecordStoreModel, "SELECT * FROM record_stores WHERE record_store_id = $1",id)
        .fetch_one(&data.db)
        .await;

    // match for error from the Result
    match query_result {
        Ok(record_store) => {
            let record_store_resp = serde_json::json!(
                {
                    "status": "success",
                    "record_store": record_store
                });

            return Ok(Json(record_store_resp));
        }
        Err(_) => {
            let error_response = serde_json::json!(
                {
                    "status": "fail",
                    "message": format!("record_store_id {} not found", id)
                });

            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

pub async fn edit_record_store(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateRecordStoreSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let query_result = sqlx::query_as!(
        RecordStoreModel,
        "SELECT * FROM record_stores WHERE record_store_id = $1", id)
        .fetch_one(&data.db)
        .await;
    
    if query_result.is_err() {
        let error_response = serde_json::json!(
            {
                "status": "fail",
                "message": format!("record store id: {} not found", id)
            });

            return  Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let record_store = query_result.unwrap();

    // modify the record store at the provided id

    let query_result = sqlx::query_as!(
            RecordStoreModel,
            "UPDATE record_stores SET store_name = $1, store_address = $2, store_city = $3, store_state = $4, 
            store_zip = $5, phone_number = $6, website = $7 WHERE record_store_id = $8 RETURNING *", 
            body.store_name.to_owned().unwrap_or(record_store.store_name),
            body.store_address.to_owned().unwrap_or(record_store.store_address),
            body.store_city.to_owned().unwrap_or(record_store.store_city),
            body.store_state.to_owned().unwrap_or(record_store.store_state),
            body.store_zip.to_owned().unwrap_or(record_store.store_zip),
            body.phone_number.to_owned().unwrap_or(record_store.phone_number.unwrap()),
            body.website.to_owned().unwrap_or(record_store.website.unwrap()),
            id,
        )
        .fetch_one(&data.db)
        .await;

    match query_result {
        // no errors -> respond with the record store
        Ok(record_store) => {
            let record_store_response = serde_json::json!(
                {
                    "status": "success",
                    "record_store": record_store
                });

            return  Ok((StatusCode::OK, Json(record_store_response)));
        }

        Err(err) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": format!("{:?}", err)})),
            ));
        }
    }

}

pub async fn delete_record_store(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let delete_query = sqlx::query!("DELETE FROM record_stores WHERE record_store_id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if delete_query == 0 {
        let error_response = serde_json::json!(
            {
                "status": "fail",
                "message": format!("record store id: {} not found", id)
            }
        );
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // assume it successfully deleted the record_store requested
    Ok(StatusCode::NO_CONTENT)
} 