use std::env;

// DB related config
mod database;

// TCP related config
mod tcp;

/// Configuration settings
#[derive(Clone, Debug)]
pub struct Config {
    pub listen_addr: String,
    pub db_max_connections: u32,
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_database: String,
    pub db_schema: String,
    pub url_base: String,
}

/// Default for config just calls basic constructor
impl Default for Config {
    fn default() -> Self {
        Self::load()
    }
}

impl Config {
    /// Load config from env vars.
    pub fn load() -> Self {
        // http server settings
        let port = env::var("HTTP_SERVER_PORT").unwrap_or("8080".into());
        let listen_addr = format!("0.0.0.0:{}", port);

        // database settings
        let db_max_connections = 10u32.max((num_cpus::get() * 2 + 1) as u32);
        let db_host = env::var("DB_HOST").expect("DB_HOST not set");
        let db_port = env::var("DB_PORT")
            .unwrap_or("5432".to_owned())
            .parse()
            .expect("DB_PORT could not be parsed");
        let db_user = env::var("DB_USER").expect("DB_USER not set");
        let db_password = env::var("DB_PASS").expect("DB_PASS not set");
        let db_database = env::var("DB_NAME").expect("DB_NAME not set");
        let db_schema = env::var("DB_SCHEMA").expect("DB_SCHEMA not set");

        // service URL
        let url_base = env::var("API_URL_BASE").unwrap_or("/gsd/api/v1".into());

        // Create config
        Self {
            listen_addr,
            db_max_connections,
            db_host,
            db_port,
            db_user,
            db_password,
            db_database,
            db_schema,
            url_base,
        }
    }
}
