use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use taskam_core::{
    init_pool, CoreError, CreateTag, CreateTask, Priority, SqliteTaskRepository, Status, Tag, Task,
    TaskFilter, TaskService, UpdateTask,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    task_service: Arc<TaskService<SqliteTaskRepository>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./taskam.db?mode=rwc".to_string());
    let pool = init_pool(&database_url).await?;
    let repository = SqliteTaskRepository::new(pool);
    let task_service = Arc::new(TaskService::new(repository));

    let state = AppState { task_service };

    // Build router
    let app = Router::new()
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route(
            "/api/tasks/:id",
            get(get_task).put(update_task).delete(delete_task),
        )
        .route("/api/tags", get(list_tags).post(create_tag))
        .route(
            "/api/tasks/:id/tags/:tag_id",
            post(add_tag_to_task).delete(remove_tag_from_task),
        )
        .route("/api/tasks/:id/tags", get(get_tags_for_task))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Server listening on http://0.0.0.0:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

// Request/Response types
#[derive(Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: Option<String>,
    priority: Priority,
    status: Status,
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    priority: Option<Priority>,
    status: Option<Status>,
}

#[derive(Deserialize)]
struct TaskQuery {
    search: Option<String>,
    priority: Option<Priority>,
    status: Option<Status>,
}

#[derive(Deserialize)]
struct CreateTagRequest {
    name: String,
    color: String,
}

// Error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Wrapper for CoreError to implement IntoResponse
struct ApiError(CoreError);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self.0 {
            CoreError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            CoreError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            CoreError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            CoreError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        ApiError(err)
    }
}

// Route handlers
async fn list_tasks(
    Query(query): Query<TaskQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let filter = TaskFilter {
        search: query.search,
        priority: query.priority,
        status: query.status,
        tag_ids: None,
    };
    let tasks = state.task_service.list_tasks(Some(filter)).await?;
    Ok(Json(tasks))
}

async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let task = CreateTask {
        title: req.title,
        description: req.description,
        priority: req.priority,
        status: req.status,
    };
    let created = state.task_service.create_task(task).await?;
    Ok(Json(created))
}

async fn get_task(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Task>, ApiError> {
    let task = state.task_service.get_task(&id).await?;
    Ok(Json(task))
}

async fn update_task(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let task = UpdateTask {
        title: req.title,
        description: req.description,
        priority: req.priority,
        status: req.status,
    };
    let updated = state.task_service.update_task(&id, task).await?;
    Ok(Json(updated))
}

async fn delete_task(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    state.task_service.delete_task(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_tags(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, ApiError> {
    let tags = state.task_service.list_tags().await?;
    Ok(Json(tags))
}

async fn create_tag(
    State(state): State<AppState>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>, ApiError> {
    let tag = CreateTag {
        name: req.name,
        color: req.color,
    };
    let created = state.task_service.create_tag(tag).await?;
    Ok(Json(created))
}

async fn add_tag_to_task(
    Path((task_id, tag_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    state
        .task_service
        .add_tag_to_task(&task_id, &tag_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_tag_from_task(
    Path((task_id, tag_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    state
        .task_service
        .remove_tag_from_task(&task_id, &tag_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_tags_for_task(
    Path(task_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    let tags = state.task_service.get_tags_for_task(&task_id).await?;
    Ok(Json(tags))
}
