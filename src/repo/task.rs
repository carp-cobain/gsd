use crate::{
    domain::{Status, Task},
    Error, Result,
};
use futures_util::TryStreamExt;
use sqlx::{
    postgres::{PgPool, PgRow},
    FromRow, Row,
};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Map sqlx rows to task domain objects.
impl FromRow<'_, PgRow> for Task {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        // Extract column values
        let id = row.try_get("id")?;
        let story_id = row.try_get("story_id")?;
        let name = row.try_get("name")?;
        let status: String = row.try_get("status")?;

        // Convert to enum type
        let status = Status::from_str(&status).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        // Task
        Ok(Self {
            id,
            story_id,
            name,
            status,
        })
    }
}

/// Concrete task related database logic
pub struct TaskRepo {
    db: Arc<PgPool>,
}

impl TaskRepo {
    /// Constructor
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    /// Get a ref to the connection pool.
    fn db_ref(&self) -> &PgPool {
        self.db.as_ref()
    }
}

impl TaskRepo {
    /// Get a task by id
    pub async fn fetch(&self, id: Uuid) -> Result<Task> {
        log::debug!("select_task: {}", id);

        let sql = r#"
            SELECT id, story_id, name, status
            FROM tasks
            WHERE id = $1
            AND deleted_at IS NULL
        "#;

        let task_option = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(self.db_ref())
            .await?;

        match task_option {
            Some(task) => Ok(task),
            None => Err(Error::NotFound {
                message: format!("task not found: {}", id),
            }),
        }
    }

    /// Select tasks for a story
    pub async fn fetch_all(&self, story_id: Uuid) -> Result<Vec<Task>> {
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

    /// Insert a new task
    pub async fn create(&self, story_id: Uuid, name: String) -> Result<Task> {
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

    /// Update task name and status.
    pub async fn update(&self, id: Uuid, name: String, status: Status) -> Result<Task> {
        log::debug!("update_task: {}, {}, {}", id, name, status);

        let sql = r#"
            UPDATE tasks
            SET name = $1, status = $2, updated_at = now()
            WHERE id = $3 AND deleted_at IS NULL
            RETURNING id, story_id, name, status
        "#;

        let task = sqlx::query_as(sql)
            .bind(name)
            .bind(status.to_string())
            .bind(id)
            .fetch_one(self.db_ref())
            .await?;

        Ok(task)
    }

    /// Delete a task by setting the deleted_at timestamp.
    pub async fn delete(&self, id: Uuid) -> Result<u64> {
        log::debug!("delete_task: {}", id);

        let sql = r#"
            UPDATE tasks SET deleted_at = now()
            WHERE id = $1
            AND deleted_at IS NULL
        "#;

        let result = sqlx::query(sql).bind(id).execute(self.db_ref()).await?;

        Ok(result.rows_affected())
    }
}
