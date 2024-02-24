#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use gsd::{api, config::Config, repo::Repo};

use axum::Router;
use sqlx::migrate::Migrator;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;

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

    // Create api
    let repo = Repo::new(Arc::clone(&pool));
    let api = api::routes(Arc::new(repo));
    let router = Router::new().nest("/gsd/api/v1", api);

    // Start server
    log::info!("Server listening on {}", config.listen_addr);
    let listener = TcpListener::bind(config.listen_addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
