use crate::Error;
use sqlx::postgres::PgPool;
use std::sync::Arc;

// Define related repo functionality in child modules
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
