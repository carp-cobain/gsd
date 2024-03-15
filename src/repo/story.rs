use crate::{domain::Story, Error, Result};
use futures_util::TryStreamExt;
use sqlx::{
    postgres::{PgPool, PgRow},
    FromRow, Row,
};
use std::sync::Arc;
use uuid::Uuid;

/// Map sqlx rows to story domain objects.
impl FromRow<'_, PgRow> for Story {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            owner: row.try_get("owner")?,
        })
    }
}

/// Concrete story related database logic
pub struct StoryRepo {
    db: Arc<PgPool>,
}

impl StoryRepo {
    /// Constructor
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    /// Get a ref to the connection pool.
    fn db_ref(&self) -> &PgPool {
        self.db.as_ref()
    }
}

impl StoryRepo {
    /// Select a story by id
    pub async fn fetch(&self, id: Uuid) -> Result<Story> {
        log::debug!("fetch: {}", id);

        let sql = r#"
            SELECT id, name, owner
            FROM stories
            WHERE id = $1
            AND deleted_at IS NULL
        "#;

        let maybe_story = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(self.db_ref())
            .await?;

        match maybe_story {
            Some(story) => Ok(story),
            None => Err(Error::NotFound {
                message: format!("story not found: {}", id),
            }),
        }
    }

    /// Select stories for an owner
    pub async fn fetch_all(&self, owner: String) -> Result<Vec<Story>> {
        log::debug!("fetch_all: {}", owner);

        let sql = r#"
            SELECT id, name, owner
            FROM stories
            WHERE owner = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
        "#;

        let mut result_set = sqlx::query(sql).bind(owner).fetch(self.db_ref());
        let mut result = Vec::new();

        while let Some(row) = result_set.try_next().await? {
            let story = Story::from_row(&row)?;
            result.push(story);
        }

        Ok(result)
    }

    /// Insert a new story
    pub async fn create(&self, name: String, owner: String) -> Result<Story> {
        log::debug!("create: {}, {}", name, owner);

        let sql = r#"
            INSERT INTO stories (name, owner)
            VALUES ($1, $2)
            RETURNING id, name, owner
        "#;

        let story = sqlx::query_as(sql)
            .bind(name)
            .bind(owner)
            .fetch_one(self.db_ref())
            .await?;

        Ok(story)
    }

    /// Update story name and owner
    pub async fn update(&self, id: Uuid, name: String, owner: String) -> Result<Story> {
        log::debug!("update_story: {}, {}, {}", id, name, owner);

        let sql = r#"
            UPDATE stories
            SET name = $1, owner = $2, updated_at = now()
            WHERE id = $3 AND deleted_at IS NULL
            RETURNING id, name, owner
        "#;

        let story = sqlx::query_as(sql)
            .bind(name)
            .bind(owner)
            .bind(id)
            .fetch_one(self.db_ref())
            .await?;

        Ok(story)
    }

    /// Delete a story and its tasks by setting the deleted_at timestamp.
    pub async fn delete(&self, id: Uuid) -> Result<u64> {
        log::debug!("delete_story: {}", id);

        let mut transaction = self.db.begin().await?;

        let delete_tasks_sql = r#"
            UPDATE tasks SET deleted_at = now()
            WHERE story_id = $1
            AND deleted_at IS NULL
        "#;
        let delete_tasks_result = sqlx::query(delete_tasks_sql)
            .bind(id)
            .execute(&mut *transaction)
            .await?;

        let delete_story_sql = r#"
            UPDATE stories SET deleted_at = now()
            WHERE id = $1
            AND deleted_at IS NULL
        "#;
        let delete_story_result = sqlx::query(delete_story_sql)
            .bind(id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(delete_tasks_result.rows_affected() + delete_story_result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::tests;

    use testcontainers::{clients::Cli, RunnableImage};
    use testcontainers_modules::postgres::Postgres;

    #[ignore]
    #[tokio::test]
    async fn integration_test() {
        // Set up postgres test container backed repo
        let docker = Cli::default();
        let image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");
        let container = docker.run(image);
        let pool = tests::setup_pg_pool(&container).await;

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

        // Assert story was deleted
        let stories = story_repo.fetch_all(owner).await.unwrap();
        assert!(stories.is_empty());
    }
}
