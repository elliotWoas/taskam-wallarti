use crate::error::CoreError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        }
    }
}

impl FromStr for Priority {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_matches('"') {
            "Low" => Ok(Priority::Low),
            "Medium" => Ok(Priority::Medium),
            "High" => Ok(Priority::High),
            _ => Err(CoreError::Validation(format!("Invalid priority: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
pub enum Status {
    Todo,
    InProgress,
    Done,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Todo => "Todo",
            Status::InProgress => "InProgress",
            Status::Done => "Done",
        }
    }
}

impl FromStr for Status {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_matches('"') {
            "Todo" => Ok(Status::Todo),
            "InProgress" => Ok(Status::InProgress),
            "Done" => Ok(Status::Done),
            _ => Err(CoreError::Validation(format!("Invalid status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[ts(export)]
pub struct TaskFilter {
    pub search: Option<String>,
    pub priority: Option<Priority>,
    pub status: Option<Status>,
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub status: Option<Status>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub color: String,
}
