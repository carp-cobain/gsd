use crate::domain::{Status, Story, Task};
use serde::Deserialize;
use std::{fmt::Debug, str::FromStr};
use uuid::Uuid;
use validator::{Validate, ValidationError};

// Min string length bytes
const MIN_LEN: u64 = 1;

// Max string length bytes
const MAX_LEN: u64 = 100;

// The query parameters for getting stories
#[derive(Debug, Deserialize, Default)]
pub struct GetStoriesParams {
    pub owner: Option<String>,
}

/// The POST body for creating stories
#[derive(Debug, Deserialize, Default, Validate)]
pub struct CreateStoryBody {
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub name: String,
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub owner: Option<String>,
}

/// The POST body for creating tasks
#[derive(Debug, Deserialize, Default, Validate)]
pub struct CreateTaskBody {
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub name: String,
    pub story_id: Uuid,
}

/// The PATCH body for updating tasks
#[derive(Debug, Deserialize, Default, Validate)]
pub struct PatchTaskBody {
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub name: Option<String>,
    #[validate(custom(function = "validate_status", message = "unmatched enum variant"))]
    pub status: Option<String>,
}

impl PatchTaskBody {
    /// Helper to unwrap fields to update for a task, falling back to existing values.
    pub fn unwrap(self, task: Task) -> (String, Status) {
        let name = self.name.unwrap_or(task.name);
        let status = match self.status {
            Some(s) => Status::from_str(&s).unwrap_or(task.status),
            None => task.status,
        };
        (name, status)
    }
}

/// The PATCH body for updating stories
#[derive(Debug, Deserialize, Default, Validate)]
pub struct PatchStoryBody {
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub name: Option<String>,
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub owner: Option<String>,
}

impl PatchStoryBody {
    /// Helper to unwrap fields to update for a story, falling back to existing values.
    pub fn unwrap(self, story: Story) -> (String, String) {
        let name = self.name.unwrap_or(story.name);
        let owner = self.owner.unwrap_or(story.owner);
        (name, owner)
    }
}

/// Custom status validation function
fn validate_status(status_opt: &Option<String>) -> Result<(), ValidationError> {
    match status_opt {
        None => Ok(()),
        Some(status) => match Status::from_str(status) {
            Err(_) => Err(ValidationError::new("invalid_status")),
            Ok(_) => Ok(()),
        },
    }
}
