use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route(
            "/api/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
}

#[derive(OpenApi)]
#[openapi(
    paths(get_tasks, create_task, get_task, update_task, delete_task),
    components(schemas(Task, CreateTask, UpdateTask, ErrorBody)),
    tags((name = "tasks", description = "Team task management"))
)]
pub struct ApiDoc;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
struct Task {
    id: String,
    team_id: String,
    title: String,
    description: Option<String>,
    status: String,
    owner_id: Option<String>,
    due_date: Option<String>,
    created_at: String,
}

#[derive(Deserialize, ToSchema)]
struct CreateTask {
    team_id: String,
    title: String,
    description: Option<String>,
    owner_id: Option<String>,
    due_date: Option<String>,
}

#[derive(Deserialize, ToSchema)]
struct UpdateTask {
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    owner_id: Option<String>,
    due_date: Option<String>,
}

#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    NotFound,
    Conflict(String),
    Database(sqlx::Error),
}

#[derive(Serialize, ToSchema)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound => (StatusCode::NOT_FOUND, "task not found".to_owned()),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Database(error) => {
                eprintln!("database error: {error}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(database_error) = &error
            && (database_error.is_foreign_key_violation() || database_error.is_unique_violation())
        {
            return Self::Conflict("task references an invalid or duplicate resource".to_owned());
        }

        Self::Database(error)
    }
}

fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() {
        return Err(ApiError::BadRequest("title cannot be empty".to_owned()));
    }
    if title.chars().count() > 200 {
        return Err(ApiError::BadRequest(
            "title cannot exceed 200 characters".to_owned(),
        ));
    }
    Ok(())
}

fn validate_status(status: &str) -> Result<(), ApiError> {
    if matches!(status, "todo" | "in_progress" | "done") {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "status must be todo, in_progress, or done".to_owned(),
        ))
    }
}

#[utoipa::path(
    get,
    path = "/api/tasks",
    tag = "tasks",
    responses(
        (status = 200, description = "Tasks ordered newest first", body = [Task]),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
async fn get_tasks(State(state): State<AppState>) -> Result<Json<Vec<Task>>, ApiError> {
    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, team_id, title, description, status, owner_id, due_date, created_at
        FROM tasks
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(tasks))
}

#[utoipa::path(
    post,
    path = "/api/tasks",
    tag = "tasks",
    request_body = CreateTask,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 400, description = "Invalid task input", body = ErrorBody),
        (status = 409, description = "Invalid team or owner", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    validate_title(&payload.title)?;
    if payload.team_id.trim().is_empty() {
        return Err(ApiError::BadRequest("team_id cannot be empty".to_owned()));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO tasks (id, team_id, title, description, status, owner_id, due_date)
        VALUES (?, ?, ?, ?, 'todo', ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&payload.team_id)
    .bind(payload.title.trim())
    .bind(&payload.description)
    .bind(&payload.owner_id)
    .bind(&payload.due_date)
    .execute(&state.db)
    .await?;

    let task = fetch_task(&state, &id).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    tag = "tasks",
    params(("id" = String, Path, description = "Task UUID")),
    responses(
        (status = 200, description = "Task found", body = Task),
        (status = 404, description = "Task not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(fetch_task(&state, &id).await?))
}

async fn fetch_task(state: &AppState, id: &str) -> Result<Task, ApiError> {
    sqlx::query_as::<_, Task>(
        r#"
        SELECT id, team_id, title, description, status, owner_id, due_date, created_at
        FROM tasks
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)
}

#[utoipa::path(
    put,
    path = "/api/tasks/{id}",
    tag = "tasks",
    params(("id" = String, Path, description = "Task UUID")),
    request_body = UpdateTask,
    responses(
        (status = 200, description = "Task updated", body = Task),
        (status = 400, description = "Invalid task input", body = ErrorBody),
        (status = 404, description = "Task not found", body = ErrorBody),
        (status = 409, description = "Invalid owner", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTask>,
) -> Result<Json<Task>, ApiError> {
    if let Some(title) = &payload.title {
        validate_title(title)?;
    }
    if let Some(status) = &payload.status {
        validate_status(status)?;
    }

    let result = sqlx::query(
        r#"
        UPDATE tasks
        SET title = COALESCE(?, title),
            description = COALESCE(?, description),
            status = COALESCE(?, status),
            owner_id = COALESCE(?, owner_id),
            due_date = COALESCE(?, due_date)
        WHERE id = ?
        "#,
    )
    .bind(payload.title.map(|title| title.trim().to_owned()))
    .bind(payload.description)
    .bind(payload.status)
    .bind(payload.owner_id)
    .bind(payload.due_date)
    .bind(&id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(fetch_task(&state, &id).await?))
}

#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    tag = "tasks",
    params(("id" = String, Path, description = "Task UUID")),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found", body = ErrorBody),
        (status = 500, description = "Database error", body = ErrorBody)
    )
)]
async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

// NOTE: write tests here later
#[cfg(test)]
mod tests {
    use super::*;
}
