use crate::{
    domain::{Status, Story, Task},
    Error,
};
use sqlx::{
    postgres::{PgPool, PgRow},
    FromRow, Row,
};
use std::{str::FromStr, sync::Arc};

// Define related queries in child modules
mod story;
mod task;

/// GSD database logic
pub struct Repo {
    db: Arc<PgPool>,
}

impl Repo {
    /// Constructor
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    /// Get a ref to the connection pool.
    fn db_ref(&self) -> &PgPool {
        self.db.as_ref()
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Internal {
            message: err.to_string(),
        }
    }
}

impl FromRow<'_, PgRow> for Story {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            story_id: row.try_get("id")?,
            name: row.try_get("name")?,
            owner: row.try_get("owner")?,
        })
    }
}

impl FromRow<'_, PgRow> for Task {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        // Extract column values
        let task_id = row.try_get("id")?;
        let story_id = row.try_get("story_id")?;
        let name = row.try_get("name")?;
        let status: String = row.try_get("status")?;

        // Convert to enum type
        let status = Status::from_str(&status).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        // Task
        Ok(Self {
            task_id,
            story_id,
            name,
            status,
        })
    }
}
