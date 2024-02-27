use crate::{
    api::dto::{CreateStoryBody, GetStoriesParams, PatchStoryBody},
    domain::Story,
    repo::Repo,
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
pub(crate) async fn get_story(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Story>> {
    log::debug!("get_story: {}", story_id);

    let story = repo.select_story(story_id).await?;
    Ok(Json(story))
}

/// Get stories by owner
pub(crate) async fn get_stories(
    params: Option<Query<GetStoriesParams>>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Vec<Story>>> {
    log::debug!("get_stories: {:?}", params);

    let Query(params) = params.unwrap_or_default();
    let owner = params.owner.unwrap_or(BACKLOG.into());

    let stories = repo.select_stories(owner).await?;
    Ok(Json(stories))
}

/// Create a new story for an owner
pub(crate) async fn create_story(
    State(repo): State<Arc<Repo>>,
    Json(body): Json<CreateStoryBody>,
) -> Result<impl IntoResponse> {
    log::debug!("create_story: {:?}", body);

    body.validate()?;

    let owner = body.owner.unwrap_or(BACKLOG.into());
    let story = repo.insert_story(body.name, owner).await?;

    Ok((StatusCode::CREATED, Json(story)))
}

/// Update a story name and/or owner.
pub(crate) async fn patch_story(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
    Json(body): Json<PatchStoryBody>,
) -> Result<Json<Story>> {
    log::debug!("patch_story: {:?}", body);

    // Validate
    let story = repo.select_story(story_id).await?;
    body.validate()?;

    // Unwrap
    let name = body.name.unwrap_or(story.name);
    let owner = body.owner.unwrap_or(story.owner);

    // Update
    let story = repo.update_story(story_id, name, owner).await?;
    Ok(Json(story))
}

/// Delete a story by id
pub(crate) async fn delete_story(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> StatusCode {
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
