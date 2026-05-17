use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::JsonValue};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::shared::queue::QueueJobType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    DeadLettered,
}

impl JobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::DeadLettered => "dead_lettered",
        }
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for JobStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "dead_lettered" => Ok(Self::DeadLettered),
            _ => Err(format!("unsupported job status: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobPayload {
    SendWelcomeEmail {
        user_id: Uuid,
        email: String,
        name: String,
    },
    ProcessUploadedFile {
        upload_id: Uuid,
        sub_folder: String,
        stored_filename: String,
        original_filename: String,
        storage_path: String,
    },
    RetryWebhook {
        target_url: String,
        reason: String,
    },
}

impl JobPayload {
    pub fn job_type(&self) -> QueueJobType {
        match self {
            Self::SendWelcomeEmail { .. } => QueueJobType::SendWelcomeEmail,
            Self::ProcessUploadedFile { .. } => QueueJobType::ProcessUploadedFile,
            Self::RetryWebhook { .. } => QueueJobType::RetryWebhook,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::SendWelcomeEmail { email, .. } => format!("welcome email for {email}"),
            Self::ProcessUploadedFile {
                sub_folder,
                original_filename,
                ..
            } => format!("process file {original_filename} in {sub_folder}"),
            Self::RetryWebhook { target_url, .. } => format!("retry webhook to {target_url}"),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub status: String,
    pub payload: JsonValue,
    pub payload_summary: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub last_error: Option<String>,
    pub created_by: Option<Uuid>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobAttempt {
    pub id: Uuid,
    pub job_id: Uuid,
    pub attempt_number: i32,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobStatusCount {
    pub status: String,
    pub total: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobTypeCount {
    pub job_type: String,
    pub total: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobDailyCount {
    pub bucket: DateTime<Utc>,
    pub succeeded: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobDurationRow {
    pub bucket: DateTime<Utc>,
    pub avg_seconds: f64,
}
