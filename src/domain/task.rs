use crate::domain::Status;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Task {
    pub task_id: Uuid,
    pub story_id: Uuid,
    pub name: String,
    pub status: Status,
}
