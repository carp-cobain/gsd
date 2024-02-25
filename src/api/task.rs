use crate::{
    api::dto::{CreateTaskBody, UpdateTaskBody},
    domain::{Status, Story, Task},
    repo::Repo,
    util::validate::Validate,
    Error,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures_util::TryFutureExt;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Get task by id
pub async fn get_task(
    Path(task_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Task>, Error> {
    log::debug!("get_task: {}", task_id);
    let task = repo.select_task(task_id).await?;
    Ok(Json(task))
}

/// Get tasks for a story
pub async fn get_tasks(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Vec<Task>>, Error> {
    log::debug!("get_tasks: {}", story_id);
    let tasks = repo.select_tasks(story_id).await?;
    Ok(Json(tasks))
}

/// Create a task new task
pub async fn create_task(
    State(repo): State<Arc<Repo>>,
    Json(body): Json<CreateTaskBody>,
) -> Result<impl IntoResponse, Error> {
    log::debug!("create_task: {:?}", body);

    let name = Validate::string_length(&body.name)?;
    let task = repo
        .select_story(body.story_id)
        .and_then(|story: Story| repo.insert_task(story.story_id, name))
        .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update a task name and/or status.
pub async fn update_task(
    Path(task_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
    Json(body): Json<UpdateTaskBody>,
) -> Result<Json<Task>, Error> {
    log::debug!("update_task: {:?}", body);

    let task = repo.select_task(task_id).await?;
    let (name, status) = unpack_task_update(task, body)?;
    let task = repo.update_task(task_id, name, status).await?;

    Ok(Json(task))
}

// Helper that determines the latest name and status for a task to update.
fn unpack_task_update(task: Task, body: UpdateTaskBody) -> Result<(String, Status), Error> {
    log::debug!("unpack_task_update");

    let tmp_name = body.name.unwrap_or(task.name);
    let name = Validate::string_length(&tmp_name)?;
    let status = match body.status {
        Some(s) => Status::from_str(&s).map_err(|err| Error::InvalidArgument {
            message: err.to_string(),
        })?,
        None => task.status,
    };

    Ok((name, status))
}

/// Delete a task by id
pub async fn delete_task(Path(task_id): Path<Uuid>, State(repo): State<Arc<Repo>>) -> StatusCode {
    log::debug!("delete_task: {}", task_id);

    let result = repo
        .select_task(task_id)
        .and_then(|task: Task| repo.delete_task(task.task_id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
