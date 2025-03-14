use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bcrypt::{hash, DEFAULT_COST};
use bigdecimal::BigDecimal;
use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use crate::{
    handlers::records::combine_supplied_genres,
    models::record::{CreateRecordSchema, RecordModel},
    models::user::{
        CreateUserSchema, FilterOptions, PutUserRecord, UpdateUserSchema, UserModel,
        UserResponseSchema,
    },
};
use crate::{models::user::PatchUserRecord, AppState};

pub async fn list_all_users(
    Query(opts): Query<FilterOptions>,
    State(data): State<Arc<AppState>>,
) -> impl IntoResponse {
    let limit = opts.limit.unwrap_or(5);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // query as the record model and return all the records
    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users ORDER BY user_name LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await;

    match query_result {
        // no error
        Ok(users) => {
            // cast the users into the UserResponseSchema
            let user_responses: Vec<UserResponseSchema> =
                users.into_iter().map(|u| u.into()).collect();

            // found records return them to client
            let json_response = serde_json::json!({
                "status": "success",
                "results": user_responses.len(),
                "users": user_responses,
            });
            println!("GET: returning users");
            (StatusCode::OK, Json(json_response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Something bad happened while fetching some users",
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

pub async fn find_specific_user(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE user_id = $1", id)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(user) => {
            // convert for security
            let converted_user: UserResponseSchema = user.into();
            let user_response = serde_json::json!({
            "status": "success",
            "user": converted_user,
            });

            println!("GET: returning details for {}", converted_user.user_name);

            return Ok(Json(user_response));
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("user_id {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

// helper function for the password hashing.
fn create_hashed_password(password_text: String) -> String {
    let hashed_password = hash(password_text, DEFAULT_COST).unwrap();

    hashed_password
}

pub async fn create_user(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    //create the insertion query into postgres
    let query_result = sqlx::query_as!(
        UserModel,
        "INSERT INTO users (user_name, user_first_name, user_last_name, user_email, user_password) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        body.user_name.to_string(),
        body.user_first_name.to_string(),
        body.user_last_name.to_string(),
        body.user_email.to_string(),
        create_hashed_password(body.user_password)
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(created_user) => {
            let converted_user: UserResponseSchema = created_user.into();
            let user_response = json!({
            "status": "success",
            "user": converted_user,
            });

            println!("POST: created user {}", converted_user.user_name);

            return Ok((StatusCode::CREATED, Json(user_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violated unique constraint")
            {
                let error_response = serde_json::json!(
                    {
                        "status": "fail",
                        "message": format!("user_name {} already exists", body.user_name),
                    }
                );
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }

            // otherwise it's not a duplicate and something went wrong
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": format!("{:?}", e)})),
            ));
        }
    }
}

pub async fn edit_user(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE user_id = $1", id)
        .fetch_one(&data.db)
        .await;

    // could be an error as in not found.
    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("user id: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // assume it can be modified from the body elements provided
    let user = query_result.unwrap();

    let query_result = sqlx::query_as!(
        UserModel,
        "UPDATE users SET user_name = $1, user_first_name = $2, user_last_name = $3, user_email = $4, user_password = $5 WHERE user_id = $6 RETURNING *",
        body.user_name.to_owned().unwrap_or(user.user_name),
        body.user_first_name.to_owned().unwrap_or(user.user_first_name),
        body.user_last_name.to_owned().unwrap_or(user.user_last_name),
        body.user_email.to_owned().unwrap_or(user.user_email),
        create_hashed_password(body.user_password.to_owned().unwrap_or(user.user_password)),
        id,
    )
    .fetch_one(&data.db)
    .await;

    // respond accordingly
    match query_result {
        // no errors
        Ok(user) => {
            let converted_user: UserResponseSchema = user.into();
            let user_response = serde_json::json!({
                "status": "success",
                "user": converted_user
            });

            println!(
                "PATCH: successfully modified {} details",
                converted_user.user_name
            );

            return Ok((StatusCode::OK, Json(user_response)));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": format!("{:?}", err)})),
            ));
        }
    }
}

/// get_user_records
/// returns all user records associated with a specific user id provided
/// get_user_records
/// returns all user records associated with a specific user id provided
pub async fn get_user_records(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // query the db
    let user_records_query: Vec<Uuid> = sqlx::query!(
        "SELECT record_id FROM user_records WHERE user_id = $1",
        user_id
    )
    .fetch_all(&data.db)
    .await
    .unwrap()
    //iterate over the returned record ids
    .into_iter()
    .map(|row| row.record_id)
    .collect();

    // show something to to client
    if user_records_query.is_empty() {
        println!("GET: returning user_id: {} records", user_id);

        return Ok(StatusCode::OK.into_response());
    }

    // query for those sweet tunes you've collected
    let record_query = sqlx::query_as!(
        RecordModel,
        "SELECT * FROM records WHERE record_id = ANY($1)",
        &user_records_query,
    )
    .fetch_all(&data.db)
    .await;

    match record_query {
        Ok(user_records) => {
            let user_records_response = json!({
                "status": "success",
                "results": user_records.len(),
                "user_records": user_records,
            });

            println!("GET: returning user_id: {} records", user_id);

            return Ok(Json(user_records_response).into_response());
        }
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("no user_records found for user id: {}", user_id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

/// create_user_records:
/// arguments:
/// * user_id: user_id associated with the new record to add
/// * State of the application: AppState
/// * record to insert : JSON
pub async fn create_user_record(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecordSchema>,
) -> impl IntoResponse {
    //query for the user if they even exist...
    let user_query_check =
        sqlx::query_as!(UserModel, "SELECT * FROM users WHERE user_id = $1", user_id)
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
                    let user_records_insert_query = sqlx::query!(
                        "INSERT INTO user_records ( user_id, record_id) VALUES ($1, $2) RETURNING user_record_id, user_id, record_id",
                        found_user.user_id,
                        created_record.record_id,
                    )
                    .fetch_one(&data.db)
                    .await;

                    match user_records_insert_query {
                        Ok(created_user_record) => {
                            let create_user_record_resp = serde_json::json!({
                                "status": "success",
                                "records_collected": "1",
                                "user_id": created_user_record.user_id,
                                "user_record_id": created_user_record.user_record_id,
                                "record": created_record,
                            });

                            println!(
                                "POST: collect '{}' by '{}' for user: {}",
                                created_record.title,
                                created_record.artist,
                                created_user_record.user_id
                            );

                            (StatusCode::OK, Json(create_user_record_resp))
                        }
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "status": "fail",
                                "message": format!("error when collecting records for user_id: {}, {}", user_id, e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                        }
                    }
                }
                // something went wrong
                Err(_) => (
                    StatusCode::CONFLICT,
                    Json(
                        json!({"status": "error", "message": format!("record {} by {} already exists", body.title, body.artist)}),
                    ),
                ),
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

pub async fn put_user_record(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<PutUserRecord>,
) -> impl IntoResponse {
    //query for the user if they even exist...
    let user_query_check =
        sqlx::query_as!(UserModel, "SELECT * FROM users WHERE user_id = $1", user_id)
            .fetch_one(&data.db)
            .await;

    // ensure the user exists
    match user_query_check {
        // yay! found a user! let's add some sweet music
        Ok(found_user) => {
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
                Ok(created_record) => {
                    // check to confirm
                    if let Ok(Some(_)) = sqlx::query!(
                        "SELECT * FROM user_records WHERE record_id = $1",
                        body.record_id
                    )
                    .fetch_optional(&data.db)
                    .await
                    {
                        let duplicate_response = json!(
                            {
                                "status": "error",
                                "message": format!("record_id: {} already in collection", body.record_id)
                            }
                        );

                        return (StatusCode::CONFLICT, Json(duplicate_response));
                    }

                    // add this to the user_records table by associated user_id
                    let user_records_insert_query = sqlx::query!(

                        "INSERT INTO user_records ( user_id, record_id) VALUES ($1, $2) RETURNING user_record_id, user_id, record_id",
                        found_user.user_id,
                        created_record.record_id,
                    )
                    .fetch_one(&data.db)
                    .await;

                    match user_records_insert_query {
                        Ok(created_user_record) => {
                            let create_user_record_resp = serde_json::json!({
                                "status": "success",
                                "records_collected": "1",
                                "user_id": created_user_record.user_id,
                                "user_record_id": created_user_record.user_record_id,
                                "record": created_record,
                            });

                            println!(
                                "PUT: added '{}' by '{}' to  user_id: {} collection ",
                                created_record.title,
                                created_record.artist,
                                created_user_record.user_id
                            );

                            (StatusCode::OK, Json(create_user_record_resp))
                        }
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "status": "fail",
                                "message": format!("error when collecting records for user_id: {}, {}", user_id, e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                        }
                    }
                }
                Err(_) => (
                    StatusCode::NOT_FOUND,
                    Json(
                        json!({"status": "error", "message": format!("record_id: {} not found", body.record_id)}),
                    ),
                ),
            }
        }
        // assume not an valid uuid
        Err(_) => {
            let error_response = serde_json::json!(
            {
                "status": "fail",
                "message": format!("user_id: {} not found", body.record_id)
            });

            return (StatusCode::NOT_FOUND, Json(error_response));
        }
    }
}

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
            "message": format!("no records found for user id: {}", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    println!("DELETE: removed user_id: {} record collection", id);

    Ok(StatusCode::NO_CONTENT) // Make sure this is inside the function and properly closed
}

// DELETE all user records
pub async fn remove_user_record(
    Path(user_id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<PatchUserRecord>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_check = sqlx::query!("SELECT user_id FROM users WHERE user_id = $1", user_id)
        .fetch_optional(&data.db)
        .await
        .unwrap();

    // checking to ensure it's not a false user
    if user_check.is_none() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("User with id {} not found", user_id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // Query for the record
    let record_check = sqlx::query!(
        "SELECT record_id FROM records WHERE record_id = $1",
        body.record_id
    )
    .fetch_optional(&data.db)
    .await
    .unwrap();

    if record_check.is_none() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Record with id {} not found", body.record_id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let rows_affected = sqlx::query!(
        "DELETE FROM user_records WHERE user_id = $1 AND record_id = $2",
        user_id,
        body.record_id
    )
    .execute(&data.db)
    .await
    .unwrap()
    .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("No user_records found for user_id: {} and record_id: {}", user_id, body.record_id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    println!(
        "PATCH: removed record_id {} from user_id: {} collection",
        body.record_id, user_id
    );

    Ok(StatusCode::NO_CONTENT)
}

/// delete_user:
/// DELETE for removing the user_id supplied for the user
/// the service intends to delete
pub async fn delete_user(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let delete_query = sqlx::query!("DELETE FROM users WHERE user_id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if delete_query == 0 {
        let error_reponse = serde_json::json!(
            {
                "status": "fail",
                "message": format!("user id: {} not found", id)
            }
        );
        return Err((StatusCode::NOT_FOUND, Json(error_reponse)));
    }

    println!("DELETE: removed user_id: {}", id);

    // assume something disappeared
    Ok(StatusCode::NO_CONTENT)
}
