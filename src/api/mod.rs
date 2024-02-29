use axum::Router;
use std::sync::Arc;

mod ctx;
mod dto;
mod story;
mod task;

pub use ctx::ApiCtx;

/// The top-level GSD web-service API
pub struct Api {
    ctx: Arc<ApiCtx>,
}

impl Api {
    /// Create a new service
    pub fn new(ctx: Arc<ApiCtx>) -> Self {
        Self { ctx }
    }

    /// Define API routes, mapping paths to handlers.
    pub fn routes(self) -> Router {
        story::routes().merge(task::routes()).with_state(self.ctx)
    }
}
