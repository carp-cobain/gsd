use crate::{
    api::{
        dto::{CreateTaskBody, PatchTaskBody},
        ApiCtx,
    },
    domain::Task,
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
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// API routes for tasks
pub fn routes() -> Router<Arc<ApiCtx>> {
    Router::new().route("/tasks", post(create_task)).route(
        "/tasks/:id",
        get(get_task).delete(delete_task).patch(update_task),
    )
}

/// Get task by id
async fn get_task(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> Result<Json<Task>> {
    log::debug!("get_task: {}", id);
    let task = ctx.task_repo.fetch(id).await?;
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
        .story_repo
        .fetch(body.story_id)
        .and_then(|_| ctx.task_repo.create(body.story_id, body.name))
        .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update a task name and/or status.
async fn update_task(
    Path(id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<PatchTaskBody>,
) -> Result<Json<Task>> {
    log::debug!("update_task: {}, {:?}", id, body);

    body.validate()?;
    let task = ctx.task_repo.fetch(id).await?;

    let (name, status) = body.unwrap(task);
    let task = ctx.task_repo.update(id, name, status).await?;

    Ok(Json(task))
}

/// Delete a task by id
async fn delete_task(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> StatusCode {
    log::debug!("delete_task: {}", id);

    let result = ctx
        .task_repo
        .fetch(id)
        .and_then(|_| ctx.task_repo.delete(id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => StatusCode::from(error),
    }
}
