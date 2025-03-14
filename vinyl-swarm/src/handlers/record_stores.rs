use std::sync::Arc;

use axum::{
    extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json
};

use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::{
    models::user::UserModel,
    models::store::{RecordStoreModel, FilterOptions, UpdateRecordStoreSchema, PatchRecordStoreSchema,  PutRecordStoreSchema, CreateRecordStoreSchema}
};


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
            println!("GET: returning all record_stores");
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

    // check for an existing record_store
    if let Ok(Some(found_store)) = sqlx::query_as!(
        RecordStoreModel,
        "SELECT * FROM record_stores WHERE store_name = $1 AND store_address = $2 AND store_city = $3 AND store_state = $4",
        body.store_name,
        body.store_address, 
        body.store_city,
        body.store_state
    ).fetch_optional(&data.db).await {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"status": "fail", "message": format!("Record store '{}' already exists.", found_store.store_name)}))
        ));

    }

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

            println!("POST: created {} record store ", created_record_store.store_name);

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

            println!("GET: returning {} record store", record_store.store_name);
    
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
            
            println!("PATCH: editing {} store details", record_store.store_name);

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
    println!("DELETE: removed record_store: {}", id);
    Ok(StatusCode::NO_CONTENT)
} 

// USER RECORD STORE ENDPOINTS: 
pub async fn get_user_record_stores(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {


    // check for the user.
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

    // query the db
    let user_stores_query: Vec<Uuid> = sqlx::query!("SELECT record_store_id FROM user_record_stores WHERE user_key = $1", user_id)
        .fetch_all(&data.db)
        .await
        .unwrap()
        //iterate over the returned record ids
        .into_iter()
        .map(|row| row.record_store_id)
        .collect();

    // show something to to client
    if user_stores_query.is_empty() {
        return Ok(StatusCode::OK.into_response());
    }

    // query for those sweet tunes you've collected
    let stores_query = sqlx::query_as!(
            RecordStoreModel,
            "SELECT * FROM record_stores WHERE record_store_id = ANY($1)",
            &user_stores_query,
        )
        .fetch_all(&data.db)
        .await;

    match stores_query {
        Ok(record_stores) => {
            // deserialize the model with serde. 
            let user_record_stores_response = serde_json::json!({
                "status": "success",
                "results": record_stores.len(),
                "user_record_stores": record_stores,
            });
            println!("GET: returning user_id: {} saved record stores", user_id);
            return Ok(Json(user_record_stores_response).into_response());
        }
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("no user_records found for user id: {}", user_id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }

    // look for the store to return.


}

pub async fn add_existing_record_store(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<PutRecordStoreSchema>
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
        // yay! found a user! add the record store now
        Ok(found_user) => {
            // query for the existing record_store
            let check_record_store_query = sqlx::query_as!(
                RecordStoreModel,
                "SELECT * FROM record_stores WHERE record_store_id = $1",
                body.record_store_id,
            )
            .fetch_one(&data.db)
            .await;

            // hopefully I get a record model back.

            match check_record_store_query {
                Ok(existing_record_store) => {
                    // add this to the user_records table by associated user_id
                    let user_record_store_insert = sqlx::query!(
                        "INSERT INTO user_record_stores ( user_key, record_store_id) VALUES ($1, $2) RETURNING user_favorite_stores_id, user_key, record_store_id",
                        found_user.user_id,
                        body.record_store_id,
                    )
                    .fetch_one(&data.db)
                    .await;

                    match user_record_store_insert {
                        Ok(user_record_store) => {
                            let user_wished_created_response = serde_json::json!({
                                "status": "success",
                                "user_id": user_record_store.user_key,
                                "user_favorite_stores_id": user_record_store.user_favorite_stores_id,
                                "record_store": existing_record_store,
                            });
                            println!("PUT: adding record_store '{}' to user_id: {} ", existing_record_store.store_name, user_id);

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
                    (StatusCode::NOT_FOUND, Json(json!({"status": "error", "message": format!("record_store_id: {} not found", body.record_store_id)})))
                }
            }
        }
        // assume not an valid uuid
        Err(_) => {
            let error_response = serde_json::json!(
                {
                    "status": "fail",
                    "message": format!("record_store {} not found", body.record_store_id)
                });
            
            return (StatusCode::NOT_FOUND, Json(error_response));
        }
    }

}

pub  async fn add_user_record_store(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecordStoreSchema>
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
        // yay! found a user! let's add an awesome shop
        Ok(found_user) => {
           // query for the new record_store insertion
            let create_record_store = sqlx::query_as!(
                RecordStoreModel,
                "INSERT INTO record_stores (store_name, store_address, store_city, store_state, store_zip, phone_number, website)
                VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
                body.store_name,
                body.store_address,
                body.store_city,
                body.store_state,
                body.store_zip,
                body.phone_number.unwrap_or("".to_string()),
                body.website.unwrap_or("".to_string()),
            )
            .fetch_one(&data.db)
            .await;

            // hopefully I get a record model back.

            match create_record_store {
                Ok(created_record_store) => {
                    // add this to the user_records table by associated user_id
                    let user_record_store_insert_query = sqlx::query!(
                        "INSERT INTO user_record_stores ( user_key, record_store_id) VALUES ($1, $2) RETURNING user_key, record_store_id",
                        found_user.user_id,
                        created_record_store.record_store_id,
                    )
                    .fetch_one(&data.db)
                    .await;

                    match user_record_store_insert_query {
                        Ok(inserted_user_store) => {
                            let created_store_response = serde_json::json!({
                                "status": "success",
                                "user_id": inserted_user_store.user_key,
                                "user_record_store_id": inserted_user_store.record_store_id,
                                "record": created_record_store,
                            });
                            println!("POST: adding new record_store: '{}' to user_id: {} collection.", created_record_store.store_name, user_id);
                            (StatusCode::OK, Json(created_store_response))
                        },
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "status": "fail",
                                "message": format!("error when creating record store for user_id: {}, {}", user_id, e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                        }
                    }
                }
                // something went wrong.
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

pub async fn delete_user_record_store(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<PatchRecordStoreSchema>,
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

    // check for the existence of the record store
    let record_store_check = sqlx::query!(
        "DELETE from user_record_stores WHERE record_store_id = $1 AND user_key = $2",
        body.record_store_id,
        user_id,
    )
    .execute(&data.db)
    .await
    .unwrap()
    .rows_affected();

    if record_store_check == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("not record stores found for user_id: {} with record_store_id: {}", user_id, body.record_store_id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    println!("DELETE: removed record_store_id '{}' from user_id: {}", body.record_store_id,  user_id);
    Ok(StatusCode::NO_CONTENT)
}