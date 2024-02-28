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
    Json,
};
use futures_util::TryFutureExt;
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;
use validator::Validate;

/// Get task by id
pub(crate) async fn get(
    Path(task_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
) -> Result<Json<Task>> {
    log::debug!("get: {}", task_id);

    let task = ctx.repo.select_task(task_id).await?;
    Ok(Json(task))
}

/// Get tasks for a story
pub(crate) async fn list(
    Path(story_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
) -> Result<Json<Vec<Task>>> {
    log::debug!("list: {}", story_id);

    let tasks = ctx.repo.select_tasks(story_id).await?;
    Ok(Json(tasks))
}

/// Create a task new task
pub(crate) async fn create(
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<CreateTaskBody>,
) -> Result<impl IntoResponse> {
    log::debug!("create: {:?}", body);

    body.validate()?;

    let task = ctx
        .repo
        .select_story(body.story_id)
        .and_then(|_| ctx.repo.insert_task(body.story_id, body.name))
        .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update a task name and/or status.
pub(crate) async fn patch(
    Path(task_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<PatchTaskBody>,
) -> Result<Json<Task>> {
    log::debug!("patch: {:?}", body);

    // Validate
    body.validate()?;
    let task = ctx.repo.select_task(task_id).await?;

    // Unwrap
    let name = body.name.unwrap_or(task.name);
    let status = match body.status {
        Some(s) => Status::from_str(&s).unwrap(), // Already validated above
        None => task.status,
    };

    // Update
    let task = ctx.repo.update_task(task_id, name, status).await?;
    Ok(Json(task))
}

/// Delete a task by id
pub(crate) async fn delete(
    Path(task_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
) -> StatusCode {
    log::debug!("delete: {}", task_id);

    let result = ctx
        .repo
        .select_task(task_id)
        .and_then(|_| ctx.repo.delete_task(task_id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
