use crate::{
    api::dto::{CreateStoryBody, GetStoriesParams, UpdateStoryBody},
    domain::Story,
    repo::Repo,
    util::validate::Validate,
    Error,
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

/// Default owner for stories
pub const BACKLOG: &str = "backlog";

/// Get story by id
pub async fn get_story(
    Path(story_id): Path<Uuid>,
    State(repo): State<Arc<Repo>>,
) -> Result<Json<Story>, Error> {
    log::debug!("get_story: {}", story_id);
    let story = repo.select_story(story_id).await?;
    Ok(Json(story))
}

/// Get stories by owner
pub async fn get_stories(
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

/// Create a new story for an owner
pub async fn create_story(
    State(repo): State<Arc<Repo>>,
    Json(body): Json<CreateStoryBody>,
) -> Result<impl IntoResponse, Error> {
    log::debug!("create_story: {:?}", body);

    let name = Validate::string_length(&body.name)?;
    let owner = Validate::string_length(&body.owner.unwrap_or(BACKLOG.into()))?;
    let story = repo.insert_story(name, owner).await?;

    Ok((StatusCode::CREATED, Json(story)))
}

/// Update a story name and/or owner.
pub async fn update_story(
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

/// Delete a story by id
pub async fn delete_story(Path(story_id): Path<Uuid>, State(repo): State<Arc<Repo>>) -> StatusCode {
    log::debug!("delete_story: {}", story_id);

    let result = repo
        .select_story(story_id)
        .and_then(|story: Story| repo.delete_story(story.story_id))
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(error) => error.into(),
    }
}
