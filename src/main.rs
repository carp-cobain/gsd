use gsd::{
    api::{Api, ApiCtx},
    config::Config,
    repo::{StoryRepo, TaskRepo},
};

use axum::Router;
use sqlx::migrate::Migrator;
use std::{error::Error, sync::Arc};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Embed migrations into the server binary.
pub static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Init logging
    env_logger::init();

    // Load config
    let config = Config::default();
    log::debug!("Loaded config = {:?}", config);

    // Create pg connection pool
    let pool = config
        .db_pool_opts()
        .connect(config.db_connection_string().as_ref())
        .await?;

    log::info!("Running migrations");
    MIGRATOR.run(&pool).await?;

    // Arc up connection pool for async sharing across tasks
    let pool = Arc::new(pool);

    // Set up repos
    let story_repo = StoryRepo::new(Arc::clone(&pool));
    let task_repo = TaskRepo::new(Arc::clone(&pool));

    // Set up API context
    let ctx = ApiCtx::new(Arc::new(story_repo), Arc::new(task_repo));

    // Set up API
    let api = Api::new(Arc::new(ctx));
    let router = Router::new().nest(&config.url_base, api.routes());

    // Start server
    log::info!("Server listening on {}", config.listen_addr);
    axum::serve(config.tcp_listener(), router).await?;

    Ok(())
}
