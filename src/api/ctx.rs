use crate::repo::Repo;
use std::sync::Arc;

// NOTE: Split repo, add drivers here

#[derive(Clone)]
pub struct ApiCtx {
    pub repo: Arc<Repo>,
}

impl ApiCtx {
    pub fn new(repo: Arc<Repo>) -> Self {
        Self { repo }
    }
}
