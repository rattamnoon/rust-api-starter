use chrono::Utc;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::modules::jobs::model::{
    Job, JobAttempt, JobDailyCount, JobDurationRow, JobPayload, JobStatus, JobStatusCount,
    JobTypeCount,
};

#[derive(Clone)]
pub struct JobRepository {
    pool: PgPool,
}

pub struct NewJob {
    pub job_type: String,
    pub payload: JobPayload,
    pub payload_summary: String,
    pub max_attempts: i32,
    pub created_by: Option<Uuid>,
}

impl JobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: NewJob) -> Result<Job, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            "INSERT INTO jobs (
                job_type, status, payload, payload_summary, attempt_count, max_attempts, created_by, queued_at
             ) VALUES ($1, $2, $3, $4, 0, $5, $6, now())
             RETURNING id, job_type, status, payload, payload_summary, attempt_count, max_attempts, last_error, created_by, queued_at, started_at, finished_at, created_at, updated_at",
        )
        .bind(input.job_type)
        .bind(JobStatus::Queued.as_str())
        .bind(Json(input.payload))
        .bind(input.payload_summary)
        .bind(input.max_attempts)
        .bind(input.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list(
        &self,
        page: i64,
        limit: i64,
        status: Option<&str>,
        job_type: Option<&str>,
    ) -> Result<Vec<Job>, sqlx::Error> {
        let offset = (page - 1) * limit;
        sqlx::query_as::<_, Job>(
            "SELECT id, job_type, status, payload, payload_summary, attempt_count, max_attempts, last_error, created_by, queued_at, started_at, finished_at, created_at, updated_at
             FROM jobs
             WHERE ($1::text IS NULL OR status = $1)
               AND ($2::text IS NULL OR job_type = $2)
             ORDER BY created_at DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(status)
        .bind(job_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            "SELECT id, job_type, status, payload, payload_summary, attempt_count, max_attempts, last_error, created_by, queued_at, started_at, finished_at, created_at, updated_at
             FROM jobs
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_attempts(&self, job_id: Uuid) -> Result<Vec<JobAttempt>, sqlx::Error> {
        sqlx::query_as::<_, JobAttempt>(
            "SELECT id, job_id, attempt_number, status, error_message, started_at, finished_at, created_at
             FROM job_attempts
             WHERE job_id = $1
             ORDER BY attempt_number ASC",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_running(&self, job_id: Uuid, attempt_number: i32) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE jobs
             SET status = $2, attempt_count = $3, started_at = COALESCE(started_at, $4), updated_at = $4
             WHERE id = $1",
        )
        .bind(job_id)
        .bind(JobStatus::Running.as_str())
        .bind(attempt_number)
        .bind(now)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO job_attempts (job_id, attempt_number, status, started_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(job_id)
        .bind(attempt_number)
        .bind(JobStatus::Running.as_str())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_succeeded(
        &self,
        job_id: Uuid,
        attempt_number: i32,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE jobs
             SET status = $2, finished_at = $3, last_error = NULL, updated_at = $3
             WHERE id = $1",
        )
        .bind(job_id)
        .bind(JobStatus::Succeeded.as_str())
        .bind(now)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE job_attempts
             SET status = $3, finished_at = $4
             WHERE job_id = $1 AND attempt_number = $2",
        )
        .bind(job_id)
        .bind(attempt_number)
        .bind(JobStatus::Succeeded.as_str())
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(
        &self,
        job_id: Uuid,
        attempt_number: i32,
        error_message: &str,
        dead_lettered: bool,
    ) -> Result<(), sqlx::Error> {
        let status = if dead_lettered {
            JobStatus::DeadLettered
        } else {
            JobStatus::Failed
        };
        let now = Utc::now();

        sqlx::query(
            "UPDATE jobs
             SET status = $2, last_error = $3, finished_at = $4, updated_at = $4
             WHERE id = $1",
        )
        .bind(job_id)
        .bind(status.as_str())
        .bind(error_message)
        .bind(now)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE job_attempts
             SET status = $3, error_message = $4, finished_at = $5
             WHERE job_id = $1 AND attempt_number = $2",
        )
        .bind(job_id)
        .bind(attempt_number)
        .bind(status.as_str())
        .bind(error_message)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn requeue(&self, job_id: Uuid) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            "UPDATE jobs
             SET status = $2, attempt_count = 0, finished_at = NULL, started_at = NULL, last_error = NULL, queued_at = now(), updated_at = now()
             WHERE id = $1
               AND status IN ($3, $4)
             RETURNING id, job_type, status, payload, payload_summary, attempt_count, max_attempts, last_error, created_by, queued_at, started_at, finished_at, created_at, updated_at",
        )
        .bind(job_id)
        .bind(JobStatus::Queued.as_str())
        .bind(JobStatus::Failed.as_str())
        .bind(JobStatus::DeadLettered.as_str())
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn count_queued(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM jobs WHERE status = $1")
            .bind(JobStatus::Queued.as_str())
            .fetch_one(&self.pool)
            .await
    }

    pub async fn status_counts(&self) -> Result<Vec<JobStatusCount>, sqlx::Error> {
        sqlx::query_as::<_, JobStatusCount>(
            "SELECT status, COUNT(*)::bigint AS total
             FROM jobs
             GROUP BY status
             ORDER BY status",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn type_counts(&self) -> Result<Vec<JobTypeCount>, sqlx::Error> {
        sqlx::query_as::<_, JobTypeCount>(
            "SELECT job_type, COUNT(*)::bigint AS total
             FROM jobs
             GROUP BY job_type
             ORDER BY job_type",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn timeline_counts(&self) -> Result<Vec<JobDailyCount>, sqlx::Error> {
        sqlx::query_as::<_, JobDailyCount>(
            "SELECT
                date_trunc('day', created_at) AS bucket,
                COUNT(*) FILTER (WHERE status = 'succeeded')::bigint AS succeeded,
                COUNT(*) FILTER (WHERE status IN ('failed', 'dead_lettered'))::bigint AS failed
             FROM jobs
             WHERE created_at >= now() - interval '30 days'
             GROUP BY bucket
             ORDER BY bucket",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn average_processing_time(&self) -> Result<Vec<JobDurationRow>, sqlx::Error> {
        sqlx::query_as::<_, JobDurationRow>(
            "SELECT
                date_trunc('day', created_at) AS bucket,
                COALESCE(AVG(EXTRACT(EPOCH FROM (finished_at - started_at))), 0)::double precision AS avg_seconds
             FROM jobs
             WHERE started_at IS NOT NULL
               AND finished_at IS NOT NULL
               AND created_at >= now() - interval '30 days'
             GROUP BY bucket
             ORDER BY bucket",
        )
        .fetch_all(&self.pool)
        .await
    }
}
