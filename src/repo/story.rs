use crate::{domain::Story, repo::Repo, Error, Result};

use futures_util::TryStreamExt;
use sqlx::FromRow;
use uuid::Uuid;

impl Repo {
    /// Get a story by id
    pub async fn select_story(&self, story_id: Uuid) -> Result<Story> {
        log::debug!("select_story: {}", story_id);

        let sql = r#"
            SELECT id, name, owner
            FROM stories
            WHERE id = $1
            AND deleted_at IS NULL
        "#;

        let maybe_story = sqlx::query_as(sql)
            .bind(story_id)
            .fetch_optional(self.db_ref())
            .await?;

        match maybe_story {
            Some(story) => Ok(story),
            None => Err(Error::NotFound {
                message: format!("story not found: {}", story_id),
            }),
        }
    }

    /// Select stories for an owner
    pub async fn select_stories(&self, owner: String) -> Result<Vec<Story>> {
        log::debug!("select_stories: {}", owner);

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
    pub async fn insert_story(&self, name: String, owner: String) -> Result<Story> {
        log::debug!("insert_story: {}, {}", name, owner);

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

    /// Update story name and owner.
    pub async fn update_story(&self, story_id: Uuid, name: String, owner: String) -> Result<Story> {
        log::debug!("update_story: {}, {}, {}", story_id, name, owner);

        let sql = r#"
            UPDATE stories
            SET name = $1, owner = $2, updated_at = now()
            WHERE id = $3
            RETURNING id, name, owner
        "#;

        let story = sqlx::query_as(sql)
            .bind(name)
            .bind(owner)
            .bind(story_id)
            .fetch_one(self.db_ref())
            .await?;

        Ok(story)
    }

    /// Delete a story and its tasks by setting the deleted_at timestamp.
    pub async fn delete_story(&self, story_id: Uuid) -> Result<u64> {
        log::debug!("delete_story: {}", story_id);

        let mut transaction = self.db.begin().await?;

        let delete_tasks_sql = r#"
            UPDATE tasks SET deleted_at = now()
            WHERE story_id = $1
            AND deleted_at IS NULL
        "#;
        let delete_tasks_result = sqlx::query(delete_tasks_sql)
            .bind(story_id)
            .execute(&mut *transaction)
            .await?;

        let delete_story_sql = r#"
            UPDATE stories SET deleted_at = now()
            WHERE id = $1
            AND deleted_at IS NULL
        "#;
        let delete_story_result = sqlx::query(delete_story_sql)
            .bind(story_id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(delete_tasks_result.rows_affected() + delete_story_result.rows_affected())
    }
}
