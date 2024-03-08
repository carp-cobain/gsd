use crate::{
    config::Config,
    repo::{StoryRepo, TaskRepo},
};
use sqlx::postgres::PgPool;
use std::sync::Arc;

// NOTE: Add drivers here

#[derive(Clone)]
pub struct ApiCtx {
    pub config: Arc<Config>,
    pub story_repo: Arc<StoryRepo>,
    pub task_repo: Arc<TaskRepo>,
}

impl ApiCtx {
    pub fn new(config: Arc<Config>, db: Arc<PgPool>) -> Self {
        Self {
            config,
            story_repo: Arc::new(StoryRepo::new(Arc::clone(&db))),
            task_repo: Arc::new(TaskRepo::new(Arc::clone(&db))),
        }
    }
}
