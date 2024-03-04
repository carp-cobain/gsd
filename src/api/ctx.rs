use crate::repo::{StoryRepo, TaskRepo};
use std::sync::Arc;

// NOTE: Add drivers here

#[derive(Clone)]
pub struct ApiCtx {
    pub story_repo: Arc<StoryRepo>,
    pub task_repo: Arc<TaskRepo>,
}

impl ApiCtx {
    pub fn new(story_repo: Arc<StoryRepo>, task_repo: Arc<TaskRepo>) -> Self {
        Self {
            story_repo,
            task_repo,
        }
    }
}
