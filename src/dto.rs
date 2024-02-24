use serde::Deserialize;
use uuid::Uuid;

// The query parameters for getting stories
#[derive(Debug, Deserialize, Default)]
pub struct GetStoriesParams {
    pub owner: Option<String>,
}

/// The POST body for creating stories
#[derive(Debug, Deserialize, Default)]
pub struct CreateStoryBody {
    pub name: String,
    pub owner: Option<String>,
}

/// The POST body for creating tasks
#[derive(Debug, Deserialize, Default)]
pub struct CreateTaskBody {
    pub name: String,
    pub story_id: Uuid,
}

/// The PATCH body for updating tasks
#[derive(Debug, Deserialize, Default)]
pub struct UpdateTaskBody {
    pub name: Option<String>,
    pub status: Option<String>,
}

/// The PATCH body for updating stories
#[derive(Debug, Deserialize, Default)]
pub struct UpdateStoryBody {
    pub name: Option<String>,
    pub owner: Option<String>,
}
