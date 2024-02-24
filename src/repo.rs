use crate::entity::{Status, Story, Task};
use crate::{Error, Result};

use futures_util::TryStreamExt;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// GSD database logic
pub struct Repo {
    db: Arc<PgPool>,
}

impl Repo {
    /// Constructor
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    /// Get a ref to the connection pool.
    fn db_ref(&self) -> &PgPool {
        self.db.as_ref()
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Internal {
            message: err.to_string(),
        }
    }
}

impl FromRow<'_, PgRow> for Story {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            story_id: row.try_get("id")?,
            name: row.try_get("name")?,
            owner: row.try_get("owner")?,
        })
    }
}

impl FromRow<'_, PgRow> for Task {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        // Extract column values
        let task_id = row.try_get("id")?;
        let story_id = row.try_get("story_id")?;
        let name = row.try_get("name")?;
        let status: String = row.try_get("status")?;

        // Convert to enum type
        let status = Status::from_str(&status).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        // Task
        Ok(Self {
            task_id,
            story_id,
            name,
            status,
        })
    }
}

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

    /// Get a task by id
    pub async fn select_task(&self, task_id: Uuid) -> Result<Task> {
        log::debug!("select_task: {}", task_id);

        let sql = r#"
            SELECT id, story_id, name, status
            FROM tasks
            WHERE id = $1
            AND deleted_at IS NULL
        "#;

        let task_option = sqlx::query_as(sql)
            .bind(task_id)
            .fetch_optional(self.db_ref())
            .await?;

        match task_option {
            Some(task) => Ok(task),
            None => Err(Error::NotFound {
                message: format!("task not found: {}", task_id),
            }),
        }
    }

    /// Insert a new task
    pub async fn insert_task(&self, story_id: Uuid, name: String) -> Result<Task> {
        log::debug!("insert_task: {}, {}", story_id, name);

        let sql = r#"
            INSERT INTO tasks (story_id, name)
            VALUES ($1, $2)
            RETURNING id, story_id, name, status
        "#;

        let task = sqlx::query_as(sql)
            .bind(story_id)
            .bind(name)
            .fetch_one(self.db_ref())
            .await?;

        Ok(task)
    }

    /// Select tasks for a story
    pub async fn select_tasks(&self, story_id: Uuid) -> Result<Vec<Task>> {
        log::debug!("select_tasks: story: {}", story_id);

        let sql = r#"
            SELECT id, story_id, name, status
            FROM tasks
            WHERE story_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
        "#;

        let mut result_set = sqlx::query(sql).bind(story_id).fetch(self.db_ref());
        let mut result = Vec::new();

        while let Some(row) = result_set.try_next().await? {
            let task = Task::from_row(&row)?;
            result.push(task);
        }

        Ok(result)
    }

    /// Update task name and status.
    pub async fn update_task(&self, task_id: Uuid, name: String, status: Status) -> Result<Task> {
        log::debug!("update_task: {}, {}, {}", task_id, name, status);

        let sql = r#"
            UPDATE tasks
            SET name = $1, status = $2, updated_at = now()
            WHERE id = $3
            RETURNING id, story_id, name, status
        "#;

        let task = sqlx::query_as(sql)
            .bind(name)
            .bind(status.to_string())
            .bind(task_id)
            .fetch_one(self.db_ref())
            .await?;

        Ok(task)
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

    /// Delete a task by setting the deleted_at timestamp.
    pub async fn delete_task(&self, task_id: Uuid) -> Result<u64> {
        log::debug!("delete_task: {}", task_id);

        let sql = r#"
            UPDATE tasks SET deleted_at = now()
            WHERE id = $1
            AND deleted_at IS NULL
        "#;

        let result = sqlx::query(sql)
            .bind(task_id)
            .execute(self.db_ref())
            .await?;

        Ok(result.rows_affected())
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
}
