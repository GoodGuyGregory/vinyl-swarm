use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use chrono::offset;
use serde_json::json;

use crate::{
    model::NoteModel,
    schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};


/// basic status check endpoint
pub async fn status_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres, and Axum";

    let json_response = serde_json::json!({
        "status": "Success",
        "message": MESSAGE
    });

    // return the response
    Json(json_response)
}



// basic GET ALL request with filtering options
pub async fn note_list_handler(
    Query(opts): Query<FilterOptions>,
    State(data): State<Arc<AppState>>,
) -> impl IntoResponse {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query_result = sqlx::query_as!(
        NoteModel,
        "SELECT * FROM notes ORDER BY id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await;

    match query_result {
        Ok(notes) => {
            let json_response = serde_json::json!({
                "status": "success",
                "results": notes.len(),
                "notes": notes,
            });
            (StatusCode::OK, Json(json_response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Something bad happened while fetching all note items",
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}


// POST to create our notes
pub async fn create_note_handler(State(data): State<Arc<AppState>>, Json(body): Json<CreateNoteSchema>,) 
-> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
    //create the insertion query into postgres
    let query_result = sqlx::query_as!(
        NoteModel, 
        "INSERT INTO notes (title, content, category) VALUES ($1, $2, $3) RETURNING *",
        body.title.to_string(),
        body.content.to_string(),
        body.category.to_owned().unwrap_or("".to_string())
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(note) => {
            let note_response = json!({
                "status": "success",
                "data": json!({
                    "note": note
                })
            });

            return Ok((StatusCode::CREATED, Json(note_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violated unique constraint") 
            {
                let error_response = serde_json::json!(
                    {
                        "status": "fail",
                        "messsage": "Note with that title already exists",
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

// GET a Single Record from the DB
pub async fn read_note_handler(Path(id): Path<uuid::Uuid>, State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let query_result = sqlx::query_as!(NoteModel, 
        "SELECT * FROM notes WHERE id = $1", id)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({
                "status": "success", 
                "data": serde_json::json!(
                    {
                        "note": note
                    })
            });

            return Ok(Json(note_response));
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}


// UPDATE an exisiting note object this will leverage the UpdateSchema for the Note Struct
pub async fn edit_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(NoteModel, "SELECT * FROM notes WHERE id = $1", id)
        .fetch_one(&data.db)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let note = query_result.unwrap();

    let query_result = sqlx::query_as!(
        NoteModel,
        "UPDATE notes SET title = $1, content = $2, category = $3, published = $4, updated_at = $5 WHERE id = $6 RETURNING *",
        body.title.to_owned().unwrap_or(note.title),
        body.content.to_owned().unwrap_or(note.content),
        body.category.to_owned().unwrap_or(note.category.unwrap()),
        body.published.unwrap_or(note.published.unwrap()),
        now,
        id
    )
    .fetch_one(&data.db)
    .await
    ;

    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": note
            })});

            return Ok(Json(note_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", err)})),
            ));
        }
    }
}


// DELETE endpoint to remove a note from the database
pub async fn delete_note_handler(Path(id): Path<uuid::Uuid>, State(data): State<Arc<AppState>>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // execute a query to delete the id from the path parameter 
    let rows_affected = sqlx::query!("DELETE FROM notes WHERE id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    // rows_affected should return a status for us to view
    if rows_affected == 0 {
        let error_response = serde_json::json!(
            {
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            }
        );
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // else we can just return nothing
    Ok(StatusCode::NO_CONTENT)
}