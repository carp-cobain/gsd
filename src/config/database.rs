use crate::config::Config;
use percent_encoding::NON_ALPHANUMERIC;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;
use std::sync::Arc;

impl Config {
    pub fn db_connection_string(&self) -> String {
        let bytes = self.db_password.as_bytes();
        let password = percent_encoding::percent_encode(bytes, NON_ALPHANUMERIC);
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.db_user, password, self.db_host, self.db_port, self.db_database,
        )
    }

    pub fn db_pool_opts(&self) -> PgPoolOptions {
        let schema = Arc::new(self.db_schema.clone());
        PgPoolOptions::new()
            .max_connections(self.db_max_connections)
            .after_connect(move |conn, _meta| {
                let schema = Arc::clone(&schema);
                Box::pin(async move {
                    conn.execute(format!("SET search_path = '{}';", schema).as_ref())
                        .await?;
                    Ok(())
                })
            })
    }
}
