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
    use super::*;
    use crate::domain::Status;
    use sqlx::{
        migrate::Migrator,
        postgres::{PgPool, PgPoolOptions},
    };
    use std::{path::Path, sync::Arc};

    use testcontainers::{clients::Cli, Container, RunnableImage};
    use testcontainers_modules::postgres::Postgres;

    async fn setup_pg_pool(container: &Container<'_, Postgres>) -> Arc<PgPool> {
        let connection_string = &format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            container.get_host_port_ipv4(5432),
        );

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&connection_string)
            .await
            .unwrap();

        log::debug!("Running migrations on test container");
        let m = Migrator::new(Path::new("./migrations")).await.unwrap();
        m.run(&pool).await.unwrap();

        Arc::new(pool)
    }

    #[ignore]
    #[tokio::test]
    async fn story_repo_integration_test() {
        // Set up postgres test container backed repo
        let docker = Cli::default();
        let image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");
        let container = docker.run(image);
        let pool = setup_pg_pool(&container).await;

        // Set up repo under test
        let story_repo = StoryRepo::new(pool);

        // Create story
        let name = "Books To Read".to_string();
        let owner = "github.com/carp-cobain".to_string();
        let story = story_repo
            .create(name.clone(), owner.clone())
            .await
            .unwrap();
        assert_eq!(name, story.name);

        // Query stories for owner
        let stories = story_repo.fetch_all(owner.clone()).await.unwrap();
        assert_eq!(stories.len(), 1);

        // Delete the story
        let rows_updated = story_repo.delete(story.id).await.unwrap();
        assert_eq!(rows_updated, 1);
    }

    #[ignore]
    #[tokio::test]
    async fn task_repo_integration_test() {
        // Set up postgres test container backed repo
        let docker = Cli::default();
        let image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");
        let container = docker.run(image);
        let pool = setup_pg_pool(&container).await;
        let story_repo = StoryRepo::new(Arc::clone(&pool));

        // Set up repo under test
        let task_repo = TaskRepo::new(Arc::clone(&pool));

        // Set up a story to put tasks under
        let name = "Books To Read".to_string();
        let owner = "github.com/carp-cobain".to_string();
        let story_id = story_repo
            .create(name.clone(), owner.clone())
            .await
            .unwrap()
            .id;

        // Create task, ensuring complete flag is false
        let task_name = "Suttree".to_string();
        let task = task_repo
            .create(story_id.clone(), task_name.clone())
            .await
            .unwrap();
        assert_eq!(task.status, Status::Incomplete);

        // Complete task
        let task = task_repo
            .update(task.id, task.name, Status::Complete)
            .await
            .unwrap();
        assert_eq!(task.status, Status::Complete);

        // Query tasks for story.
        let tasks = task_repo.fetch_all(story_id).await.unwrap();
        assert_eq!(tasks.len(), 1);

        // Delete the task
        let updated_rows = task_repo.delete(task.id).await.unwrap();
        assert_eq!(updated_rows, 1);
    }
}
