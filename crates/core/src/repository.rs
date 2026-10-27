use super::error::CoreError;
use super::models::{CreateTag, CreateTask, Priority, Status, Tag, Task, TaskFilter, UpdateTask};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Row, Sqlite};
use std::str::FromStr;

#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create_task(&self, task: CreateTask) -> Result<Task, CoreError>;
    async fn get_task(&self, id: &str) -> Result<Task, CoreError>;
    async fn update_task(&self, id: &str, task: UpdateTask) -> Result<Task, CoreError>;
    async fn delete_task(&self, id: &str) -> Result<(), CoreError>;
    async fn list_tasks(&self, filter: Option<TaskFilter>) -> Result<Vec<Task>, CoreError>;
    async fn create_tag(&self, tag: CreateTag) -> Result<Tag, CoreError>;
    async fn list_tags(&self) -> Result<Vec<Tag>, CoreError>;
    async fn add_tag_to_task(&self, task_id: &str, tag_id: &str) -> Result<(), CoreError>;
    async fn remove_tag_from_task(&self, task_id: &str, tag_id: &str) -> Result<(), CoreError>;
    async fn get_tags_for_task(&self, task_id: &str) -> Result<Vec<Tag>, CoreError>;
}

pub struct SqliteTaskRepository {
    pool: Pool<Sqlite>,
}

impl SqliteTaskRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TaskRepository for SqliteTaskRepository {
    async fn create_task(&self, task: CreateTask) -> Result<Task, CoreError> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let completed_at = if task.status == Status::Done {
            Some(created_at)
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, description, priority, status, created_at, completed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.priority.as_str())
        .bind(task.status.as_str())
        .bind(created_at.to_rfc3339())
        .bind(completed_at.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await?;

        self.get_task(&id).await
    }

    async fn get_task(&self, id: &str) -> Result<Task, CoreError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, description, priority, status, created_at, completed_at
            FROM tasks
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| CoreError::NotFound(format!("Task with id {} not found", id)))?;

        Ok(Task {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            priority: Priority::from_str(&row.get::<String, _>("priority"))
                .unwrap_or(Priority::Medium),
            status: Status::from_str(&row.get::<String, _>("status"))
                .unwrap_or(Status::Todo),
            created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
            completed_at: row
                .get::<Option<String>, _>("completed_at")
                .map(|t| DateTime::parse_from_rfc3339(&t).unwrap().with_timezone(&Utc)),
        })
    }

    async fn update_task(&self, id: &str, task: UpdateTask) -> Result<Task, CoreError> {
        let current = self.get_task(id).await?;

        let title = task.title.unwrap_or(current.title);
        let description = task.description.or(current.description);
        let priority = task.priority.unwrap_or(current.priority);
        let status = task.status.unwrap_or(current.status);

        let completed_at = if status == Status::Done && current.completed_at.is_none() {
            Some(Utc::now())
        } else if status != Status::Done {
            None
        } else {
            current.completed_at
        };

        sqlx::query(
            r#"
            UPDATE tasks
            SET title = ?1, description = ?2, priority = ?3, status = ?4, completed_at = ?5
            WHERE id = ?6
            "#,
        )
        .bind(&title)
        .bind(&description)
        .bind(priority.as_str())
        .bind(status.as_str())
        .bind(completed_at.map(|t| t.to_rfc3339()))
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_task(id).await
    }

    async fn delete_task(&self, id: &str) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!("Task with id {} not found", id)));
        }

        Ok(())
    }

    async fn list_tasks(&self, filter: Option<TaskFilter>) -> Result<Vec<Task>, CoreError> {
        let filter = filter.unwrap_or_default();
        let mut query = String::from(
            "SELECT id, title, description, priority, status, created_at, completed_at FROM tasks WHERE 1=1",
        );
        let mut params = Vec::new();

        if let Some(search) = &filter.search {
            query.push_str(" AND (title LIKE ? OR description LIKE ?)");
            let pattern = format!("%{}%", search);
            params.push(pattern.clone());
            params.push(pattern);
        }

        if let Some(priority) = &filter.priority {
            query.push_str(" AND priority = ?");
            params.push(priority.as_str().to_string());
        }

        if let Some(status) = &filter.status {
            query.push_str(" AND status = ?");
            params.push(status.as_str().to_string());
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query);
        for param in params {
            q = q.bind(param);
        }

        let rows = q.fetch_all(&self.pool).await?;

        let tasks = rows
            .into_iter()
            .map(|row| Task {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                priority: Priority::from_str(&row.get::<String, _>("priority"))
                    .unwrap_or(Priority::Medium),
                status: Status::from_str(&row.get::<String, _>("status"))
                    .unwrap_or(Status::Todo),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                completed_at: row
                    .get::<Option<String>, _>("completed_at")
                    .map(|t| DateTime::parse_from_rfc3339(&t).unwrap().with_timezone(&Utc)),
            })
            .collect();

        Ok(tasks)
    }

    async fn create_tag(&self, tag: CreateTag) -> Result<Tag, CoreError> {
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO tags (id, name, color)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&id)
        .bind(&tag.name)
        .bind(&tag.color)
        .execute(&self.pool)
        .await?;

        Ok(Tag {
            id,
            name: tag.name,
            color: tag.color,
        })
    }

    async fn list_tags(&self) -> Result<Vec<Tag>, CoreError> {
        let rows = sqlx::query("SELECT id, name, color FROM tags ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| Tag {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
            })
            .collect())
    }

    async fn add_tag_to_task(&self, task_id: &str, tag_id: &str) -> Result<(), CoreError> {
        sqlx::query("INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)")
            .bind(task_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn remove_tag_from_task(&self, task_id: &str, tag_id: &str) -> Result<(), CoreError> {
        sqlx::query("DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2")
            .bind(task_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_tags_for_task(&self, task_id: &str) -> Result<Vec<Tag>, CoreError> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.name, t.color
            FROM tags t
            INNER JOIN task_tags tt ON t.id = tt.tag_id
            WHERE tt.task_id = ?1
            ORDER BY t.name
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Tag {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
            })
            .collect())
    }
}
