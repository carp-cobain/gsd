use gsd::{
    api::{Api, ApiCtx},
    config::Config,
};

use axum::Router;
use dotenv::dotenv;
use sqlx::migrate::Migrator;
use std::{error::Error, sync::Arc};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Embed migrations into the server binary.
pub static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load env vars, init logging
    dotenv().ok();
    env_logger::init();

    // Load config
    let config = Arc::new(Config::default());
    log::debug!("Loaded config = {:?}", config);

    // Create pg connection pool
    let pool = config
        .db_pool_opts()
        .connect(config.db_connection_string().as_ref())
        .await?;

    log::info!("Running migrations");
    MIGRATOR.run(&pool).await?;

    // Set up API
    let ctx = ApiCtx::new(Arc::clone(&config), Arc::new(pool));
    let api = Api::new(Arc::new(ctx));
    let router = Router::new().nest(&config.url_base, api.routes());

    // Start server
    log::info!("Server listening on {}", config.listen_addr);
    axum::serve(config.tcp_listener(), router).await?;

    Ok(())
}
