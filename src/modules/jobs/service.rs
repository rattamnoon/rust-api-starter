use std::time::Instant;

use chrono::Utc;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::{
        jobs::{
            dto::{
                JobDetailResponse, JobResponse, JobsChartSummaryResponse, JobsListResponse,
                JobsQuery,
            },
            model::JobPayload,
            repository::{JobRepository, NewJob},
        },
        receipts::service::ReceiptService,
        uploads::model::UploadedFile,
        users::model::User,
    },
    shared::{
        extractor::AuthenticatedUser,
        metrics,
        queue::{QueueJobMessage, QueueJobType, RabbitMqClient},
        types::UserRole,
    },
};

#[derive(Clone)]
pub struct JobService {
    repository: JobRepository,
    queue: RabbitMqClient,
    max_retries: i32,
}

#[derive(Clone)]
pub struct WorkerJobService {
    repository: JobRepository,
    queue: RabbitMqClient,
    receipt_service: ReceiptService,
    max_retries: i32,
}

impl JobService {
    pub fn new(repository: JobRepository, queue: RabbitMqClient, max_retries: i32) -> Self {
        Self {
            repository,
            queue,
            max_retries,
        }
    }

    pub async fn enqueue_welcome_email(
        &self,
        user: &User,
        created_by: Option<Uuid>,
    ) -> Result<JobResponse, AppError> {
        let payload = JobPayload::SendWelcomeEmail {
            user_id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
        };
        self.enqueue(payload, created_by).await
    }

    pub async fn enqueue_uploaded_file_processing(
        &self,
        file: &UploadedFile,
    ) -> Result<JobResponse, AppError> {
        let payload = JobPayload::ProcessUploadedFile {
            upload_id: file.id,
            sub_folder: file.sub_folder.clone(),
            stored_filename: file.stored_filename.clone(),
            original_filename: file.original_filename.clone(),
            storage_path: file.storage_path.clone(),
        };
        self.enqueue(payload, Some(file.uploaded_by)).await
    }

    pub async fn enqueue_receipt_generation(
        &self,
        order_id: Uuid,
        external_event_id: &str,
        created_by: Option<Uuid>,
    ) -> Result<JobResponse, AppError> {
        self.enqueue(
            JobPayload::GenerateReceiptPdf {
                order_id,
                external_event_id: external_event_id.to_string(),
            },
            created_by,
        )
        .await
    }

    pub async fn list_jobs(
        &self,
        actor: &AuthenticatedUser,
        query: JobsQuery,
    ) -> Result<JobsListResponse, AppError> {
        ensure_admin(actor)?;
        let query = query.normalized()?;
        let jobs = self
            .repository
            .list(
                query.page,
                query.limit,
                query.status.as_deref(),
                query.job_type.as_deref(),
            )
            .await?;
        let items = jobs.into_iter().map(JobResponse::from).collect();
        Ok(JobsListResponse {
            items,
            page: query.page,
            limit: query.limit,
        })
    }

    pub async fn get_job(
        &self,
        actor: &AuthenticatedUser,
        job_id: Uuid,
    ) -> Result<JobDetailResponse, AppError> {
        ensure_admin(actor)?;
        let job = self
            .repository
            .find_by_id(job_id)
            .await?
            .ok_or_else(|| AppError::NotFound("job was not found".into()))?;
        let attempts = self.repository.list_attempts(job_id).await?;
        Ok(JobDetailResponse {
            job: JobResponse::from(job),
            attempts: attempts.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn retry_job(
        &self,
        actor: &AuthenticatedUser,
        job_id: Uuid,
    ) -> Result<JobResponse, AppError> {
        ensure_admin(actor)?;
        let job = self.repository.requeue(job_id).await?.ok_or_else(|| {
            AppError::BadRequest("only failed or dead-lettered jobs can be retried".into())
        })?;

        let message = QueueJobMessage {
            job_id: job.id,
            job_type: parse_job_type(&job.job_type)?,
            attempt: 1,
            created_at: Utc::now(),
            trace_id: None,
        };
        self.queue.publish_job(&message).await?;
        metrics::record_job_retried(&job.job_type);
        metrics::record_job_published(&job.job_type);
        Ok(JobResponse::from(job))
    }

    pub async fn chart_summary(
        &self,
        actor: &AuthenticatedUser,
    ) -> Result<JobsChartSummaryResponse, AppError> {
        ensure_admin(actor)?;
        let queued_jobs = self.repository.count_queued().await?;
        metrics::set_queue_depth(queued_jobs);

        Ok(JobsChartSummaryResponse {
            by_status: self
                .repository
                .status_counts()
                .await?
                .into_iter()
                .map(Into::into)
                .collect(),
            by_type: self
                .repository
                .type_counts()
                .await?
                .into_iter()
                .map(Into::into)
                .collect(),
            timeline: self
                .repository
                .timeline_counts()
                .await?
                .into_iter()
                .map(Into::into)
                .collect(),
            average_processing_time: self
                .repository
                .average_processing_time()
                .await?
                .into_iter()
                .map(Into::into)
                .collect(),
            queued_jobs,
        })
    }

    async fn enqueue(
        &self,
        payload: JobPayload,
        created_by: Option<Uuid>,
    ) -> Result<JobResponse, AppError> {
        let job_type = payload.job_type();
        let job = self
            .repository
            .create(NewJob {
                job_type: job_type.as_str().to_string(),
                payload_summary: payload.summary(),
                payload,
                max_attempts: self.max_retries,
                created_by,
            })
            .await?;

        let message = QueueJobMessage {
            job_id: job.id,
            job_type: job_type.clone(),
            attempt: 1,
            created_at: Utc::now(),
            trace_id: None,
        };
        self.queue.publish_job(&message).await?;
        metrics::record_job_published(job_type.as_str());
        Ok(JobResponse::from(job))
    }
}

impl WorkerJobService {
    pub fn new(
        repository: JobRepository,
        queue: RabbitMqClient,
        receipt_service: ReceiptService,
        max_retries: i32,
    ) -> Self {
        Self {
            repository,
            queue,
            receipt_service,
            max_retries,
        }
    }

    pub async fn process_message(&self, message: QueueJobMessage) -> Result<(), AppError> {
        let job = self
            .repository
            .find_by_id(message.job_id)
            .await?
            .ok_or_else(|| AppError::NotFound("job was not found".into()))?;

        let payload: JobPayload = serde_json::from_value(job.payload.clone())
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let started_at = Instant::now();

        self.repository
            .mark_running(job.id, message.attempt)
            .await?;
        metrics::record_job_consumed(&job.job_type);
        metrics::job_started(&job.job_type);

        let result = self.execute_payload(&payload).await;
        let processing_seconds = started_at.elapsed().as_secs_f64();
        metrics::job_finished(&job.job_type);

        match result {
            Ok(()) => {
                self.repository
                    .mark_succeeded(job.id, message.attempt)
                    .await?;
                metrics::record_job_succeeded(&job.job_type, processing_seconds);
                Ok(())
            }
            Err(error) => {
                let can_retry = message.attempt < self.max_retries;
                if can_retry {
                    self.repository
                        .mark_failed(job.id, message.attempt, &error.to_string(), false)
                        .await?;
                    metrics::record_job_failed(&job.job_type, processing_seconds);
                    metrics::record_job_retried(&job.job_type);
                    self.queue
                        .publish_job(&QueueJobMessage {
                            attempt: message.attempt + 1,
                            ..message
                        })
                        .await?;
                } else {
                    self.repository
                        .mark_failed(job.id, message.attempt, &error.to_string(), true)
                        .await?;
                    metrics::record_job_failed(&job.job_type, processing_seconds);
                    self.queue.publish_dead_letter(&message).await?;
                }
                Err(error)
            }
        }
    }
}

impl WorkerJobService {
    async fn execute_payload(&self, payload: &JobPayload) -> Result<(), AppError> {
        match payload {
            JobPayload::SendWelcomeEmail { email, name, .. } => {
                tracing::info!("processed welcome email job for {name} <{email}>");
                Ok(())
            }
            JobPayload::ProcessUploadedFile {
                upload_id,
                storage_path,
                ..
            } => {
                tokio::fs::metadata(storage_path)
                    .await
                    .map_err(AppError::from)?;
                tracing::info!("processed uploaded file job for upload {upload_id}");
                Ok(())
            }
            JobPayload::GenerateReceiptPdf {
                order_id,
                external_event_id,
            } => {
                let receipt = self
                    .receipt_service
                    .generate_receipt_pdf(*order_id, external_event_id)
                    .await?;

                let job = self
                    .repository
                    .create(NewJob {
                        job_type: QueueJobType::SendReceiptEmail.as_str().to_string(),
                        payload_summary: JobPayload::SendReceiptEmail {
                            receipt_id: receipt.id,
                        }
                        .summary(),
                        payload: JobPayload::SendReceiptEmail {
                            receipt_id: receipt.id,
                        },
                        max_attempts: self.max_retries,
                        created_by: None,
                    })
                    .await?;
                self.queue
                    .publish_job(&QueueJobMessage {
                        job_id: job.id,
                        job_type: QueueJobType::SendReceiptEmail,
                        attempt: 1,
                        created_at: Utc::now(),
                        trace_id: None,
                    })
                    .await?;
                metrics::record_job_published(QueueJobType::SendReceiptEmail.as_str());
                Ok(())
            }
            JobPayload::SendReceiptEmail { receipt_id } => {
                let _receipt = self.receipt_service.send_receipt_email(*receipt_id).await?;
                Ok(())
            }
            JobPayload::RetryWebhook { target_url, reason } => {
                tracing::info!("processed retry webhook placeholder for {target_url}: {reason}");
                Ok(())
            }
        }
    }
}

fn parse_job_type(value: &str) -> Result<crate::shared::queue::QueueJobType, AppError> {
    match value {
        "send_welcome_email" => Ok(crate::shared::queue::QueueJobType::SendWelcomeEmail),
        "process_uploaded_file" => Ok(crate::shared::queue::QueueJobType::ProcessUploadedFile),
        "generate_receipt_pdf" => Ok(crate::shared::queue::QueueJobType::GenerateReceiptPdf),
        "send_receipt_email" => Ok(crate::shared::queue::QueueJobType::SendReceiptEmail),
        "retry_webhook" => Ok(crate::shared::queue::QueueJobType::RetryWebhook),
        _ => Err(AppError::Internal(format!("unsupported job type: {value}"))),
    }
}

fn ensure_admin(actor: &AuthenticatedUser) -> Result<(), AppError> {
    if actor.role == UserRole::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin access is required".into()))
    }
}
