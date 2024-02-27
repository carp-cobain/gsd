use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Story {
    #[serde(rename = "id")]
    pub story_id: Uuid,
    pub name: String,
    pub owner: String,
}
