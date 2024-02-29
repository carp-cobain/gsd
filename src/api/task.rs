use crate::{
    api::{
        dto::{CreateTaskBody, PatchTaskBody},
        ApiCtx,
    },
    domain::{Status, Task},
    Result,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::TryFutureExt;
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;
use validator::Validate;

/// API routes for stories
pub fn routes() -> Router<Arc<ApiCtx>> {
    let tasks_id = get(task).delete(delete_task).patch(update_task);
    Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks/:id", tasks_id)
}

/// Get task by id
async fn task(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> Result<Json<Task>> {
    log::debug!("task: {}", id);
    let task = ctx.repo.select_task(id).await?;
    Ok(Json(task))
}

/// Create a task new task
async fn create_task(
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<CreateTaskBody>,
) -> Result<impl IntoResponse> {
    log::debug!("create_task: {:?}", body);

    body.validate()?;

    let task = ctx
        .repo
        .select_story(body.story_id)
        .and_then(|_| ctx.repo.insert_task(body.story_id, body.name))
        .await?;

    let result = (StatusCode::CREATED, Json(task));

    Ok(result)
}

/// Update a task name and/or status.
async fn update_task(
    Path(id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<PatchTaskBody>,
) -> Result<Json<Task>> {
    log::debug!("update_task: {}, {:?}", id, body);

    // Validate
    body.validate()?;
    let task = ctx.repo.select_task(id).await?;

    // Unwrap
    let name = body.name.unwrap_or(task.name);
    let status = match body.status {
        Some(s) => Status::from_str(&s).unwrap_or(task.status),
        None => task.status,
    };

    // Update
    let task = ctx.repo.update_task(id, name, status).await?;

    Ok(Json(task))
}

/// Delete a task by id
async fn delete_task(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> StatusCode {
    log::debug!("delete_task: {}", id);

    let result = ctx
        .repo
        .select_task(id)
        .and_then(|_| ctx.repo.delete_task(id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
