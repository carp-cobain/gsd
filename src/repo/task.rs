use crate::{
    domain::{Status, Task},
    repo::Repo,
    Error, Result,
};

use futures_util::TryStreamExt;
use sqlx::FromRow;
use uuid::Uuid;

impl Repo {
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
}