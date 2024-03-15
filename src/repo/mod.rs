use crate::Error;

mod story;
mod task;

pub use story::StoryRepo;
pub use task::TaskRepo;

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Internal {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{
        migrate::Migrator,
        postgres::{PgPool, PgPoolOptions},
    };
    use std::{path::Path, sync::Arc};

    use testcontainers::Container;
    use testcontainers_modules::postgres::Postgres;

    /// Given a running Postgres container, set up a connection pool and run migrations.
    pub async fn setup_pg_pool<'a>(container: &Container<'a, Postgres>) -> Arc<PgPool> {
        let connection_string = &format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            container.get_host_port_ipv4(5432),
        );

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .min_connections(1)
            .connect(&connection_string)
            .await
            .unwrap();

        log::debug!("Running migrations on test container");
        let m = Migrator::new(Path::new("./migrations")).await.unwrap();
        m.run(&pool).await.unwrap();

        Arc::new(pool)
    }
}
