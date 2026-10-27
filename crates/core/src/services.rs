use super::error::CoreError;
use super::models::{CreateTag, CreateTask, Tag, Task, TaskFilter, UpdateTask};
use super::repository::TaskRepository;
use std::sync::Arc;

pub struct TaskService<R: TaskRepository> {
    repository: Arc<R>,
}

impl<R: TaskRepository> TaskService<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn create_task(&self, task: CreateTask) -> Result<Task, CoreError> {
        if task.title.trim().is_empty() {
            return Err(CoreError::Validation("Title cannot be empty".to_string()));
        }
        self.repository.create_task(task).await
    }

    pub async fn get_task(&self, id: &str) -> Result<Task, CoreError> {
        self.repository.get_task(id).await
    }

    pub async fn update_task(&self, id: &str, task: UpdateTask) -> Result<Task, CoreError> {
        if let Some(ref title) = task.title {
            if title.trim().is_empty() {
                return Err(CoreError::Validation("Title cannot be empty".to_string()));
            }
        }
        self.repository.update_task(id, task).await
    }

    pub async fn delete_task(&self, id: &str) -> Result<(), CoreError> {
        self.repository.delete_task(id).await
    }

    pub async fn list_tasks(&self, filter: Option<TaskFilter>) -> Result<Vec<Task>, CoreError> {
        self.repository.list_tasks(filter).await
    }

    pub async fn create_tag(&self, tag: CreateTag) -> Result<Tag, CoreError> {
        if tag.name.trim().is_empty() {
            return Err(CoreError::Validation("Tag name cannot be empty".to_string()));
        }
        self.repository.create_tag(tag).await
    }

    pub async fn list_tags(&self) -> Result<Vec<Tag>, CoreError> {
        self.repository.list_tags().await
    }

    pub async fn add_tag_to_task(&self, task_id: &str, tag_id: &str) -> Result<(), CoreError> {
        self.repository.add_tag_to_task(task_id, tag_id).await
    }

    pub async fn remove_tag_from_task(&self, task_id: &str, tag_id: &str) -> Result<(), CoreError> {
        self.repository.remove_tag_from_task(task_id, tag_id).await
    }

    pub async fn get_tags_for_task(&self, task_id: &str) -> Result<Vec<Tag>, CoreError> {
        self.repository.get_tags_for_task(task_id).await
    }
}
