use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::jobs::model::{
        Job, JobAttempt, JobDailyCount, JobDurationRow, JobStatus, JobStatusCount, JobTypeCount,
    },
};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct JobsQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub status: Option<String>,
    pub job_type: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobResponse {
    pub id: Uuid,
    pub job_type: String,
    pub status: String,
    pub payload: serde_json::Value,
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

impl From<Job> for JobResponse {
    fn from(value: Job) -> Self {
        Self {
            id: value.id,
            job_type: value.job_type,
            status: value.status,
            payload: value.payload,
            payload_summary: value.payload_summary,
            attempt_count: value.attempt_count,
            max_attempts: value.max_attempts,
            last_error: value.last_error,
            created_by: value.created_by,
            queued_at: value.queued_at,
            started_at: value.started_at,
            finished_at: value.finished_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobAttemptResponse {
    pub id: Uuid,
    pub attempt_number: i32,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<JobAttempt> for JobAttemptResponse {
    fn from(value: JobAttempt) -> Self {
        Self {
            id: value.id,
            attempt_number: value.attempt_number,
            status: value.status,
            error_message: value.error_message,
            started_at: value.started_at,
            finished_at: value.finished_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobDetailResponse {
    pub job: JobResponse,
    pub attempts: Vec<JobAttemptResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobsListResponse {
    pub items: Vec<JobResponse>,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChartPoint {
    pub label: String,
    pub value: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TimelinePoint {
    pub bucket: DateTime<Utc>,
    pub succeeded: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DurationPoint {
    pub bucket: DateTime<Utc>,
    pub avg_seconds: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobsChartSummaryResponse {
    pub by_status: Vec<ChartPoint>,
    pub by_type: Vec<ChartPoint>,
    pub timeline: Vec<TimelinePoint>,
    pub average_processing_time: Vec<DurationPoint>,
    pub queued_jobs: i64,
}

impl JobsQuery {
    pub fn normalized(self) -> Result<Self, AppError> {
        if let Some(status) = self.status.as_deref() {
            let _ = status
                .parse::<JobStatus>()
                .map_err(|error| AppError::BadRequest(error.to_string()))?;
        }

        Ok(Self {
            page: self.page.max(1),
            limit: self.limit.clamp(1, 100),
            status: self.status,
            job_type: self.job_type,
        })
    }
}

impl From<JobStatusCount> for ChartPoint {
    fn from(value: JobStatusCount) -> Self {
        Self {
            label: value.status,
            value: value.total,
        }
    }
}

impl From<JobTypeCount> for ChartPoint {
    fn from(value: JobTypeCount) -> Self {
        Self {
            label: value.job_type,
            value: value.total,
        }
    }
}

impl From<JobDailyCount> for TimelinePoint {
    fn from(value: JobDailyCount) -> Self {
        Self {
            bucket: value.bucket,
            succeeded: value.succeeded,
            failed: value.failed,
        }
    }
}

impl From<JobDurationRow> for DurationPoint {
    fn from(value: JobDurationRow) -> Self {
        Self {
            bucket: value.bucket,
            avg_seconds: value.avg_seconds,
        }
    }
}
