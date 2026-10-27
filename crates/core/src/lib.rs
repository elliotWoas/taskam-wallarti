pub mod error;
pub mod models;
pub mod repository;
pub mod services;

pub use error::CoreError;
pub use models::{CreateTag, CreateTask, Priority, Status, Tag, Task, TaskFilter, UpdateTask};
pub use repository::{SqliteTaskRepository, TaskRepository};
pub use services::TaskService;

use sqlx::{Pool, Sqlite};
use std::path::Path;

pub type DatabasePool = Pool<Sqlite>;

pub async fn init_pool(database_url: &str) -> Result<DatabasePool, CoreError> {
    let pool = sqlx::SqlitePool::connect(database_url).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

pub async fn init_pool_from_path(path: &Path) -> Result<DatabasePool, CoreError> {
    let database_url = format!("sqlite:{}", path.display());
    init_pool(&database_url).await
}

async fn run_migrations(pool: &DatabasePool) -> Result<(), CoreError> {
    let migration_sql = include_str!("../migrations/001_initial.sql");

    // Split by semicolon but be careful with multiline statements
    let statements: Vec<&str> = migration_sql
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }

    Ok(())
}
