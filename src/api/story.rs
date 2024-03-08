use crate::{
    api::{
        dto::{CreateStoryBody, GetStoriesParams, PatchStoryBody},
        ApiCtx,
    },
    domain::{Story, Task},
    Result,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::TryFutureExt;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Default owner for stories
const BACKLOG: &str = "backlog";

/// API routes for stories
pub fn routes() -> Router<Arc<ApiCtx>> {
    Router::new()
        .route("/stories", get(get_stories).post(create_story))
        .route("/stories/:id/tasks", get(get_tasks))
        .route(
            "/stories/:id",
            get(get_story).delete(delete_story).patch(update_story),
        )
}

/// Get story by id
async fn get_story(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> Result<Json<Story>> {
    log::debug!("get_story: {}", id);
    let story = ctx.story_repo.fetch(id).await?;
    Ok(Json(story))
}

/// Get stories by owner
async fn get_stories(
    params: Option<Query<GetStoriesParams>>,
    State(ctx): State<Arc<ApiCtx>>,
) -> Result<Json<Vec<Story>>> {
    log::debug!("get_stories: {:?}", params);

    let Query(params) = params.unwrap_or_default();
    let owner = params.owner.unwrap_or(BACKLOG.into());

    let stories = ctx.story_repo.fetch_all(owner).await?;
    Ok(Json(stories))
}

/// Get tasks for a story
async fn get_tasks(
    Path(story_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
) -> Result<Json<Vec<Task>>> {
    log::debug!("get_tasks: story_id = {}", story_id);

    let tasks = ctx
        .story_repo
        .fetch(story_id)
        .and_then(|_| ctx.task_repo.fetch_all(story_id))
        .await?;

    Ok(Json(tasks))
}

/// Create a new story for an owner
async fn create_story(
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<CreateStoryBody>,
) -> Result<impl IntoResponse> {
    log::debug!("create_story: {:?}", body);

    body.validate()?;

    let owner = body.owner.unwrap_or(BACKLOG.into());
    let story = ctx.story_repo.create(body.name, owner).await?;

    let result = (StatusCode::CREATED, Json(story));
    Ok(result)
}

/// Update a story name and/or owner.
async fn update_story(
    Path(id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<PatchStoryBody>,
) -> Result<Json<Story>> {
    log::debug!("update_story: {}, {:?}", id, body);

    body.validate()?;
    let story = ctx.story_repo.fetch(id).await?;

    let (name, owner) = body.unwrap(story);
    let story = ctx.story_repo.update(id, name, owner).await?;

    Ok(Json(story))
}

/// Delete a story by id
async fn delete_story(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> StatusCode {
    log::debug!("delete_story: {}", id);

    let result = ctx
        .story_repo
        .fetch(id)
        .and_then(|_| ctx.story_repo.delete(id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
