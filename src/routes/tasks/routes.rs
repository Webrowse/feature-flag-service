use axum::{Json, extract::{State, Path}, http::StatusCode, response::IntoResponse};
use uuid::Uuid;
use crate::state::AppState;
use crate::routes::middleware_auth::JwtUser;
use super::dto::{CreateTask, UpdateTask};
use super::queries;

pub async fn create(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Json(body): Json<CreateTask>,
) -> impl IntoResponse {
    match queries::create_task(&state.db, user_id, &body.title).await {
        Ok(t) => (StatusCode::CREATED, Json(t)).into_response(),
        Err(e) => {
            eprintln!("Error creating task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create task").into_response()
        }
    }
}

pub async fn list(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
) -> impl IntoResponse {
    match queries::list_tasks(&state.db, user_id).await {
        Ok(tasks) => (StatusCode::OK, Json(tasks)).into_response(),
        Err(e) => {
            eprintln!("Error listing tasks: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list tasks").into_response()
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTask>,
) -> impl IntoResponse {
    match queries::update_task(&state.db, user_id, id, body.title, body.done).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(e) => {
            eprintln!("Error updating task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update task").into_response()
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match queries::delete_task(&state.db, user_id, id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"deleted": true}))
        ).into_response(),
        Err(e) => {
            eprintln!("Error deleting task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete task").into_response()
        }
    }
}
