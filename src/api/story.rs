use crate::{
    api::{
        dto::{CreateStoryBody, GetStoriesParams, PatchStoryBody},
        ApiCtx,
    },
    domain::Story,
    Result,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures_util::TryFutureExt;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Default owner for stories
const BACKLOG: &str = "backlog";

/// Get story by id
pub(crate) async fn get(
    Path(story_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
) -> Result<Json<Story>> {
    log::debug!("get: {}", story_id);

    let story = ctx.repo.select_story(story_id).await?;
    Ok(Json(story))
}

/// Get stories by owner
pub(crate) async fn list(
    params: Option<Query<GetStoriesParams>>,
    State(ctx): State<Arc<ApiCtx>>,
) -> Result<Json<Vec<Story>>> {
    log::debug!("list: {:?}", params);

    let Query(params) = params.unwrap_or_default();
    let owner = params.owner.unwrap_or(BACKLOG.into());

    let stories = ctx.repo.select_stories(owner).await?;
    Ok(Json(stories))
}

/// Create a new story for an owner
pub(crate) async fn create(
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<CreateStoryBody>,
) -> Result<impl IntoResponse> {
    log::debug!("create: {:?}", body);

    body.validate()?;

    let owner = body.owner.unwrap_or(BACKLOG.into());
    let story = ctx.repo.insert_story(body.name, owner).await?;

    Ok((StatusCode::CREATED, Json(story)))
}

/// Update a story name and/or owner.
pub(crate) async fn patch(
    Path(story_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
    Json(body): Json<PatchStoryBody>,
) -> Result<Json<Story>> {
    log::debug!("patch: {:?}", body);

    // Validate
    body.validate()?;
    let story = ctx.repo.select_story(story_id).await?;

    // Unwrap
    let name = body.name.unwrap_or(story.name);
    let owner = body.owner.unwrap_or(story.owner);

    // Update
    let story = ctx.repo.update_story(story_id, name, owner).await?;
    Ok(Json(story))
}

/// Delete a story by id
pub(crate) async fn delete(
    Path(story_id): Path<Uuid>,
    State(ctx): State<Arc<ApiCtx>>,
) -> StatusCode {
    log::debug!("delete: {}", story_id);

    let result = ctx
        .repo
        .select_story(story_id)
        .and_then(|_| ctx.repo.delete_story(story_id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
