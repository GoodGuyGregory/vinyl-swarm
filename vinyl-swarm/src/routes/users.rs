use std::sync::Arc;
use bcrypt::{DEFAULT_COST, hash};

use axum::{
    http::StatusCode,
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};

use serde_json::json;
use uuid::Uuid;

use crate::AppState;
use crate::{
    models::user::{
        FilterOptions, 
        UserModel, 
        UserResponseSchema,
        UpdateUserSchema,
        CreateUserSchema,
    },
    models::record::{
        RecordModel,
    }
};

pub async fn list_all_users(
    Query(opts): Query<FilterOptions>,
    State(data): State<Arc<AppState>>
) -> impl IntoResponse {
    let limit = opts.limit.unwrap_or(5);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // query as the record model and return all the records
    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users ORDER BY user_name LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    ).fetch_all(&data.db)
    .await;

    match query_result {
        // no error
        Ok(users) => {
            // cast the users into the UserResponseSchema
            let user_responses: Vec<UserResponseSchema> = users.into_iter().map(|u| u.into()).collect();
            
            // found records return them to client
            let json_response = serde_json::json!({
                "status": "success",
                "results": user_responses.len(),
                "users": user_responses,
            });
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


pub async fn find_specific_user(Path(id): Path<Uuid>, State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let query_result = sqlx::query_as!(
        UserModel, 
        "SELECT * FROM users WHERE user_id = $1", id)
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


pub async fn create_user(State(data): State<Arc<AppState>>, Json(body): Json<CreateUserSchema>,) 
-> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
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
                Json(json!({"status": "error", "message": format!("{:?}", e)}))
            ));
        }
    }
}

pub async fn edit_user(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE user_id = $1", id)
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
            
            return Ok((StatusCode::OK, Json(user_response)));
        }
        Err(err) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR,
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
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
    // query the db
    let user_records_query: Vec<Uuid> = sqlx::query!("SELECT record_id FROM user_records WHERE user_id = $1", user_id)
        .fetch_all(&data.db)
        .await
        .unwrap()
        //iterate over the returned record ids
        .into_iter()
        .map(|row| row.record_id)
        .collect();

    // show something to to client
    if user_records_query.is_empty() {
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

// pub async fn create_user_records() {

// }


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
            "message": format!("no records found for user id: {}", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT) // Make sure this is inside the function and properly closed
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

    // assume something disappeared
    Ok(StatusCode::NO_CONTENT)
}