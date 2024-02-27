use crate::{
    api::dto::{CreateTaskBody, PatchTaskBody},
    domain::{Status, Task},
    repo::Repo,
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
pub(crate) async fn get_task(
    Path(task_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Task>> {
    log::debug!("get_task: {}", task_id);

    let task = repo.select_task(task_id).await?;
    Ok(Json(task))
}

/// Get tasks for a story
pub(crate) async fn get_tasks(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Vec<Task>>> {
    log::debug!("get_tasks: {}", story_id);

    let tasks = repo.select_tasks(story_id).await?;
    Ok(Json(tasks))
}

/// Create a task new task
pub(crate) async fn create_task(
    State(repo): State<Arc<Repo>>,
    Json(body): Json<CreateTaskBody>,
) -> Result<impl IntoResponse> {
    log::debug!("create_task: {:?}", body);

    body.validate()?;

    let task = repo
        .select_story(body.story_id)
        .and_then(|story| repo.insert_task(story.story_id, body.name))
        .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update a task name and/or status.
pub(crate) async fn patch_task(
    Path(task_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
    Json(body): Json<PatchTaskBody>,
) -> Result<Json<Task>> {
    log::debug!("patch_task: {:?}", body);

    // Validate
    let task = repo.select_task(task_id).await?;
    body.validate()?;

    // Unwrap
    let name = body.name.unwrap_or(task.name);
    let status = match body.status {
        Some(s) => Status::from_str(&s).unwrap(), // Already validated above
        None => task.status,
    };

    // Update
    let task = repo.update_task(task_id, name, status).await?;
    Ok(Json(task))
}

/// Delete a task by id
pub(crate) async fn delete_task(
    Path(task_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> StatusCode {
    log::debug!("delete_task: {}", task_id);

    let result = repo
        .select_task(task_id)
        .and_then(|task| repo.delete_task(task.task_id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
