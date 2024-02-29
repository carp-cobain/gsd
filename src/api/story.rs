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
    let stories_id = get(story).delete(delete_story).patch(update_story);
    Router::new()
        .route("/stories", get(stories).post(create_story))
        .route("/stories/:id", stories_id)
        .route("/stories/:id/tasks", get(tasks))
}

/// Get story by id
async fn story(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> Result<Json<Story>> {
    log::debug!("story: {}", id);
    let story = ctx.repo.select_story(id).await?;
    Ok(Json(story))
}

/// Get stories by owner
async fn stories(
    params: Option<Query<GetStoriesParams>>,
    State(ctx): State<Arc<ApiCtx>>,
) -> Result<Json<Vec<Story>>> {
    log::debug!("stories: {:?}", params);

    let Query(params) = params.unwrap_or_default();
    let owner = params.owner.unwrap_or(BACKLOG.into());

    let stories = ctx.repo.select_stories(owner).await?;

    Ok(Json(stories))
}

/// Get tasks for a story
async fn tasks(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> Result<Json<Vec<Task>>> {
    log::debug!("tasks: story_id = {}", id);
    let tasks = ctx.repo.select_tasks(id).await?;
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
    let story = ctx.repo.insert_story(body.name, owner).await?;

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

    // Validate
    body.validate()?;
    let story = ctx.repo.select_story(id).await?;

    // Unwrap
    let name = body.name.unwrap_or(story.name);
    let owner = body.owner.unwrap_or(story.owner);

    // Update
    let story = ctx.repo.update_story(id, name, owner).await?;

    Ok(Json(story))
}

/// Delete a story by id
async fn delete_story(Path(id): Path<Uuid>, State(ctx): State<Arc<ApiCtx>>) -> StatusCode {
    log::debug!("delete_story: {}", id);

    let result = ctx
        .repo
        .select_story(id)
        .and_then(|_| ctx.repo.delete_story(id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
