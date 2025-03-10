use std::sync::Arc;

use bigdecimal::BigDecimal;
use uuid::Uuid;
use axum::{
    body::{self, Body}, extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, routing::get, Json
};

use chrono::offset;
use serde_json::json;

use crate::{models::record::{CreateRecordSchema, UpdateRecordSchema}, AppState};
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

pub async fn find_record(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> 
{

    // get the record assuming the provided Id is valid
    let query_result = sqlx::query_as!(
        RecordModel, "SELECT * FROM records WHERE record_id = $1", id)
        .fetch_one(&data.db)
        .await;

    // match the potential error from the Result
    match query_result {
        Ok(record) => {
            let record_response = serde_json::json!(
            {
                "status": "success",
                "record": record
            });

            return Ok(Json(record_response));
        }
        Err(_) => {
            let error_response = serde_json::json!(
                {
                    "status": "fail",
                    "message": format!("record_id {} not found", id)
                });
            
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}


/// edit_record
/// deref leaves the original values in place as needed for options and passes the values
/// within the struct attributes
pub async fn edit_record(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateRecordSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let query_result = sqlx::query_as!(
        RecordModel,
        "SELECT * FROM records WHERE record_id = $1", id)
        .fetch_one(&data.db)
        .await;
    
    if query_result.is_err() {
        let error_response = serde_json::json!(
            {
                "status": "fail",
                "message": format!("record id: {} not found", id)
            });

            return  Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let record = query_result.unwrap();

    // modify the record store at the provided id

    let query_result = sqlx::query_as!(
            RecordModel,
            "UPDATE records SET artist = $1, title = $2, released = $3, genre = $4, 
            format = $5, price = $6, label = $7, duration_length = $8 WHERE record_id = $9 RETURNING *", 
            body.artist.to_owned().unwrap_or(record.artist),
            body.title.to_owned().unwrap_or(record.title),
            body.released.to_owned().unwrap_or(record.released),
            &combine_supplied_genres(body.genre),
            body.format.as_deref().unwrap_or(record.format.as_deref().unwrap_or("LP")),
            body.price.unwrap_or(record.price.unwrap_or(BigDecimal::from(0))),  
            body.label.unwrap_or(record.label),
            body.duration_length.unwrap_or(record.duration_length),
            id,
        )
        .fetch_one(&data.db)
        .await;

    match query_result {
        // no errors -> respond with the record store
        Ok(record) => {
            let record_response = serde_json::json!(
                {
                    "status": "success",
                    "record": record
                });

            return  Ok((StatusCode::OK, Json(record_response)));
        }

        Err(err) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": format!("{:?}", err)})),
            ));
        }
    }

}

/// combine_supplied_genres:
/// unwraps the Vec<String> and takes it's values.
/// or supplies and empty string vec if it fails
/// join supplies the commas for PostgreSQL Array
fn combine_supplied_genres(record_genres: Option<Vec<String>>) -> Vec<String> {
    record_genres
        .unwrap_or_else(|| vec![])  
}

/// POST add another record:
/// uses the user's id to insert a record and add's it to the user's
/// collection.
/// params: user_id, body: contains record struct
pub async fn create_new_record(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecordSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    // query for the record
    let query_result = sqlx::query_as!(
        RecordModel,
        "INSERT INTO records (artist, title, released, genre, format, price, label, duration_length)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
        body.artist.to_string(),
        body.title.to_string(),
        body.released,
        &combine_supplied_genres(body.genre),
        // unwrap if not supplied
        body.format.as_deref().unwrap_or("LP"),
        // if not supplied create empty value
        body.price.unwrap_or(BigDecimal::from(0)),
        body.label.to_string(),
        body.duration_length
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(created_record) => {
            let record_response = json!(
                {
                    "status": "success",
                    "record": created_record
                }
            );

            return Ok((StatusCode::CREATED, Json(record_response)))
        }

        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": format!("{:?}", e)}))
            ));
        }
    }

}

// DELETE specific record by id
pub async fn delete_record_by_id(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let delete_query = sqlx::query!("DELETE FROM records WHERE record_id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if delete_query == 0 {
        let error_response = serde_json::json!(
            {
                "status": "fail",
                "message": format!("record id: {} not found", id)
            }
        );
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // assume it successfully deleted the record_store requested
    Ok(StatusCode::NO_CONTENT)
} 

// DELETE all user records 
pub async fn remove_all_user_records(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let rows_affected = sqlx::query!("DELETE FROM user_records WHERE user_id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("No records found for user id: {}", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT) // Make sure this is inside the function and properly closed
} 

