use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

mod ctx;
mod dto;
mod story;
mod task;

pub use ctx::ApiCtx;

/// The top-level GSD web-service
pub struct Service {
    ctx: Arc<ApiCtx>,
}

impl Service {
    /// Create a new service
    pub fn new(ctx: Arc<ApiCtx>) -> Self {
        Self { ctx }
    }

    /// Define API routes, mapping paths to handlers.
    pub fn routes(self) -> Router {
        Router::new()
            .route("/stories", get(story::list).post(story::create))
            .route(
                "/stories/:id",
                get(story::get).delete(story::delete).patch(story::patch),
            )
            .route("/stories/:id/tasks", get(task::list))
            .route("/tasks", post(task::create))
            .route(
                "/tasks/:id",
                get(task::get).delete(task::delete).patch(task::patch),
            )
            .with_state(self.ctx)
    }
}
