use crate::{
    dto::{CreateStoryBody, CreateTaskBody, GetStoriesParams, UpdateStoryBody, UpdateTaskBody},
    entity::{Status, Story, Task},
    repo::Repo,
    validate::Validate,
    Error,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::TryFutureExt;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Default owner for stories
const BACKLOG: &'static str = "backlog";

/// Define API routes, mapping paths to handlers.
pub fn routes(repo: Arc<Repo>) -> Router {
    Router::new()
        .route("/stories", get(get_stories).post(create_story))
        .route(
            "/stories/:id",
            get(get_story).delete(delete_story).patch(update_story),
        )
        .route("/stories/:id/tasks", get(get_tasks))
        .route("/tasks", post(create_task))
        .route(
            "/tasks/:id",
            get(get_task).delete(delete_task).patch(update_task),
        )
        .with_state(repo)
}

/// Get stories by owner
async fn get_stories(
    params: Option<Query<GetStoriesParams>>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Vec<Story>>, StatusCode> {
    log::debug!("get_stories: {:?}", params);

    let Query(params) = params.unwrap_or_default();
    let owner_param = params.owner.unwrap_or(BACKLOG.into());

    let owner = Validate::string_length(&owner_param)?;
    let stories = repo.select_stories(owner).await?;

    Ok(Json(stories))
}

/// Get story by id
async fn get_story(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Story>, Error> {
    log::debug!("get_story: {}", story_id);
    let story = repo.select_story(story_id).await?;
    Ok(Json(story))
}

/// Get tasks for a story
async fn get_tasks(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Vec<Task>>, Error> {
    log::debug!("get_tasks: {}", story_id);
    let tasks = repo.select_tasks(story_id).await?;
    Ok(Json(tasks))
}

/// Create a new story for an owner
async fn create_story(
    State(repo): State<Arc<Repo>>,
    Json(body): Json<CreateStoryBody>,
) -> Result<impl IntoResponse, Error> {
    log::debug!("create_story: {:?}", body);

    let name = Validate::string_length(&body.name)?;
    let owner = Validate::string_length(&body.owner.unwrap_or(BACKLOG.into()))?;
    let story = repo.insert_story(name, owner).await?;

    Ok((StatusCode::CREATED, Json(story)))
}

/// Delete a story by id
async fn delete_story(Path(story_id): Path<Uuid>, State(repo): State<Arc<Repo>>) -> StatusCode {
    log::debug!("delete_story: {}", story_id);

    let result = repo
        .select_story(story_id)
        .and_then(|story| repo.delete_story(story.story_id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}

/// Create a task new task
async fn create_task(
    State(repo): State<Arc<Repo>>,
    Json(body): Json<CreateTaskBody>,
) -> Result<impl IntoResponse, Error> {
    log::debug!("create_task: {:?}", body);

    let name = Validate::string_length(&body.name)?;
    let task = repo
        .select_story(body.story_id)
        .and_then(|story| repo.insert_task(story.story_id, name))
        .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

/// Get task by id
async fn get_task(
    Path(task_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Task>, Error> {
    log::debug!("get_task: {}", task_id);
    let task = repo.select_task(task_id).await?;
    Ok(Json(task))
}

/// Delete a task by id
async fn delete_task(Path(task_id): Path<Uuid>, State(repo): State<Arc<Repo>>) -> StatusCode {
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

/// Update a task name and/or status.
async fn update_task(
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

/// Update a story name and/or owner.
async fn update_story(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
    Json(body): Json<UpdateStoryBody>,
) -> Result<Json<Story>, Error> {
    log::debug!("update_story: {:?}", body);

    let story = repo.select_story(story_id).await?;
    let (name, owner) = unpack_story_update(story, body)?;
    let story = repo.update_story(story_id, name, owner).await?;

    Ok(Json(story))
}

// Helper that determines the latest name and owner for a story to update.
fn unpack_story_update(story: Story, body: UpdateStoryBody) -> Result<(String, String), Error> {
    log::debug!("unpack_story_update");

    let tmp_name = body.name.unwrap_or(story.name);
    let tmp_owner = body.owner.unwrap_or(story.owner);
    let name = Validate::string_length(&tmp_name)?;
    let owner = Validate::string_length(&tmp_owner)?;

    Ok((name, owner))
}
