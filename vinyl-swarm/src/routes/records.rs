use std::sync::Arc;

use bigdecimal::BigDecimal;
use uuid::Uuid;
use axum::{ 
    extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json
};

use serde_json::json;

use crate::{
    models::{record::{CreateRecordSchema, FilterOptions, RecordModel, UpdateRecordSchema}, 
    user::{PutUserRecord, PatchUserRecord, UserModel}},
    AppState
};

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
    State(data): State<Arc<AppState>>,
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
pub fn combine_supplied_genres(record_genres: Option<Vec<String>>) -> Vec<String> {
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

    // query for the new record insertion
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

// WISHLIST ENDPOINTS:

pub async fn get_users_wishlist(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // query the db
    let user_wishlist_query: Vec<Uuid> = sqlx::query!("SELECT record_id FROM user_wishlist WHERE user_id = $1", user_id)
        .fetch_all(&data.db)
        .await
        .unwrap()
        //iterate over the returned record ids
        .into_iter()
        .map(|row| row.record_id)
        .collect();

    // show something to to client
    // basically there are no wishlist records.
    if user_wishlist_query.is_empty() {
        return Ok(StatusCode::OK.into_response());
    }

    // query for tunes users dream of owning on vinyl
    let record_query = sqlx::query_as!(
            RecordModel,
            "SELECT * FROM records WHERE record_id = ANY($1)",
            &user_wishlist_query,
        )
        .fetch_all(&data.db)
        .await;

    match record_query {
        Ok(wishlist_records) => {
            let user_wishlist_response = json!({
                "status": "success",
                "results": wishlist_records.len(),
                "user_wishlist_records": wishlist_records,
            });
            return Ok(Json(user_wishlist_response).into_response());
        }
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("no user_wishlist records found for user id: {}", user_id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

/// takes a whole record and adds items as a wish list
pub async fn add_to_user_wishlist(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecordSchema>
) -> impl IntoResponse {

     //query for the user if they even exist...
    let user_query_check = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(&data.db)
    .await;

    // ensure the user exists 
    match user_query_check {
        // yay! found a user! let's add some sweet music
        Ok(found_user) => {
            // query for the new record insertion
            let create_record_result = sqlx::query_as!(
                RecordModel,
                "INSERT INTO records (artist, title, released, genre, format, price, label, duration_length)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
                body.artist,
                body.title,
                body.released,
                &combine_supplied_genres(body.genre),
                // unwrap if not supplied
                body.format.as_deref().unwrap_or("LP"),
                // if not supplied create empty value
                body.price.unwrap_or(BigDecimal::from(0)),
                body.label,
                body.duration_length
            )
            .fetch_one(&data.db)
            .await;

            // hopefully I get a record model back.

            match create_record_result {
                Ok(created_record) => {
                    // add this to the user_records table by associated user_id
                    let user_wishlist_record = sqlx::query!(
                        "INSERT INTO user_wishlist ( user_id, record_id) VALUES ($1, $2) RETURNING user_wish_list_id, user_id, record_id, added_at",
                        found_user.user_id,
                        created_record.record_id,
                    )
                    .fetch_one(&data.db)
                    .await;

                    match user_wishlist_record {
                        Ok(created_wish_list_record) => {
                            let created_wishlist_response = serde_json::json!({
                                "status": "success",
                                "records_collected": "1",
                                "user_id": created_wish_list_record.user_id,
                                "user_record_id": created_wish_list_record.record_id,
                                "record": created_record,
                            });
                            (StatusCode::OK, Json(created_wishlist_response))
                        },
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "status": "fail",
                                "message": format!("error when collecting wishlist items for user_id: {}, {}", user_id, e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                        }
                    }
                }
                // Error finding user or something went wrong
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "error", "message": format!("{:?}", e)})))
                }
            }
        }
        // assume not an valid uuid
        Err(_) => {
            let error_response = serde_json::json!(
                {
                    "status": "fail",
                    "message": format!("user_id {} not found", user_id)
                });
            
            return (StatusCode::NOT_FOUND, Json(error_response));
        }
    }
}


pub async fn put_wishlist_record(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<PutUserRecord>,
) -> impl IntoResponse {
    
    //query for the user if they even exist...
    let user_query_check = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(&data.db)
    .await;


    // ensure the user exists 
    match user_query_check {
        // yay! found a user! let's add some sweet music
        Ok(found_user) => {
            println!("SELECTING Record id:");
            // query for the existing record
            let query_record_result = sqlx::query_as!(
                RecordModel,
                "SELECT * FROM records WHERE record_id = $1",
                body.record_id,
            )
            .fetch_one(&data.db)
            .await;

            // hopefully I get a record model back.

            match query_record_result {
                Ok(wished_record) => {
                    // add this to the user_records table by associated user_id
                    let user_wishlist_record_query = sqlx::query!(
                        "INSERT INTO user_wishlist ( user_id, record_id) VALUES ($1, $2) RETURNING user_id, record_id, user_wish_list_id",
                        found_user.user_id,
                        wished_record.record_id,
                    )
                    .fetch_one(&data.db)
                    .await;

                    match user_wishlist_record_query {
                        Ok(wished_user_record) => {
                            let user_wished_created_response = serde_json::json!({
                                "status": "success",
                                "records_collected": "1",
                                "user_id": wished_user_record.user_id,
                                "user_wish_list_id": wished_user_record.user_wish_list_id,
                                "record": wished_record,
                            });

                            (StatusCode::OK, Json(user_wished_created_response))
                        },
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "status": "fail",
                                "message": format!("error when collecting record for user_id: {}, {}", user_id, e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                        }
                    }
                }
                Err(_) => {
                    (StatusCode::NOT_FOUND, Json(json!({"status": "error", "message": format!("record_id: {} not found", body.record_id)})))
                }
            }
        }
        // assume not an valid uuid
        Err(_) => {
            let error_response = serde_json::json!(
                {
                    "status": "fail",
                    "message": format!("record {} not found", body.record_id)
                });
            
            return (StatusCode::NOT_FOUND, Json(error_response));
        }
    }

}

// DELETE a specific record from the wishlist
pub async fn remove_wishlist_record(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<PatchUserRecord>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let check_user_query = sqlx::query!(
        "SELECT user_id FROM users WHERE user_id = $1",
        user_id)
        .fetch_optional(&data.db)
        .await
        .unwrap();

    // check to ensure the user even exists
    if check_user_query.is_none() {
        let error_response = serde_json::json!(
            {
                "status":"fail",
                "message": format!("user_id {} not found", user_id)
            }
        );
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // check for the existence of the wishlist record
    let wish_list_record_check = sqlx::query!(
        "DELETE from user_wishlist WHERE record_id = $1 AND user_id = $2",
        body.record_id,
        user_id,
    )
    .execute(&data.db)
    .await
    .unwrap()
    .rows_affected();

    if wish_list_record_check == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("No wish_lists record found for user_id: {} with record_id: {}", user_id, body.record_id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)

}


pub async fn remove_user_wishlist(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("🚮 removing user's records wishlist");
    
    // checking for the user id. cause, something has to be done with an actual person
    let rows_affected = sqlx::query!("DELETE FROM user_wishlist WHERE user_id = $1", user_id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("no records found for user id: {}", user_id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    println!("user: {} user_wishlist cleared", user_id);
    Ok(StatusCode::NO_CONTENT) 
}