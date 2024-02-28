use gsd::{
    api::{ApiCtx, Service},
    config::Config,
    repo::Repo,
};

use axum::Router;
use sqlx::migrate::Migrator;
use std::{error::Error, sync::Arc};
use tokio::net::TcpListener;

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

    // Set up service
    let repo = Repo::new(Arc::clone(&pool));
    let ctx = ApiCtx::new(Arc::new(repo));
    let service = Service::new(Arc::new(ctx));

    // Set up API
    let routes = service.routes();
    let router = Router::new().nest(&config.url_base, routes);

    // Start server
    log::info!("Server listening on {}", config.listen_addr);
    let listener = TcpListener::bind(config.listen_addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
