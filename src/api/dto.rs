use crate::domain::Status;
use serde::Deserialize;
use std::{fmt::Debug, str::FromStr};
use uuid::Uuid;
use validator::{Validate, ValidationError};

// Min string length bytes
const MIN_LEN: i32 = 1;

// Max string length bytes
const MAX_LEN: i32 = 100;

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

/// The PATCH body for updating stories
#[derive(Debug, Deserialize, Default, Validate)]
pub struct PatchStoryBody {
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub name: Option<String>,
    #[validate(length(min = "MIN_LEN", max = "MAX_LEN", message = "invalid length"))]
    pub owner: Option<String>,
}

/// Custom status validation function
fn validate_status(status: &str) -> Result<(), ValidationError> {
    match Status::from_str(status) {
        Err(_) => Err(ValidationError::new("invalid_status")),
        Ok(_) => Ok(()),
    }
}
