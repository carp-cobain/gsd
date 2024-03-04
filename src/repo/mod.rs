use crate::Error;

mod story;
mod task;

pub use story::StoryRepo;
pub use task::TaskRepo;

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Internal {
            message: err.to_string(),
        }
    }
}
