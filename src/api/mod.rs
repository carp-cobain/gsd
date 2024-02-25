use crate::repo::Repo;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use story::{create_story, delete_story, get_stories, get_story, update_story};
use task::{create_task, delete_task, get_task, get_tasks, update_task};

mod dto;
mod story;
mod task;

/// Define API routes, mapping paths to handlers.
pub fn routes(repo: Arc<Repo>) -> Router {
    Router::new()
        .route("/stories", get(get_stories).post(create_story))
        .route(
            "/stories/:id",
            get(get_story).delete(delete_story).patch(update_story),
        )
        .route("/stories/:id/tasks", get(get_tasks))
        .route("/tasks", post(create_task))
        .route(
            "/tasks/:id",
            get(get_task).delete(delete_task).patch(update_task),
        )
        .with_state(repo)
}
