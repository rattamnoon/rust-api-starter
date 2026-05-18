use chrono::Utc;
use serde_json::Value;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::modules::events::model::{DomainEvent, EventPublishStatus};

#[derive(Clone)]
pub struct EventRepository {
    pool: PgPool,
}

pub struct NewDomainEvent {
    pub topic: String,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: Value,
}

impl EventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: NewDomainEvent) -> Result<DomainEvent, sqlx::Error> {
        sqlx::query_as::<_, DomainEvent>(
            "INSERT INTO domain_events (
                topic, aggregate_type, aggregate_id, event_type, payload, publish_status
             ) VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, topic, aggregate_type, aggregate_id, event_type, payload, publish_status, published_at, last_error, created_at, updated_at",
        )
        .bind(input.topic)
        .bind(input.aggregate_type)
        .bind(input.aggregate_id)
        .bind(input.event_type)
        .bind(Json(input.payload))
        .bind(EventPublishStatus::Pending.as_str())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list(
        &self,
        page: i64,
        limit: i64,
        status: Option<&str>,
        topic: Option<&str>,
    ) -> Result<Vec<DomainEvent>, sqlx::Error> {
        let offset = (page - 1).max(0) * limit;
        sqlx::query_as::<_, DomainEvent>(
            "SELECT id, topic, aggregate_type, aggregate_id, event_type, payload, publish_status, published_at, last_error, created_at, updated_at
             FROM domain_events
             WHERE ($1::text IS NULL OR publish_status = $1)
               AND ($2::text IS NULL OR topic = $2)
             ORDER BY created_at DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(status)
        .bind(topic)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DomainEvent>, sqlx::Error> {
        sqlx::query_as::<_, DomainEvent>(
            "SELECT id, topic, aggregate_type, aggregate_id, event_type, payload, publish_status, published_at, last_error, created_at, updated_at
             FROM domain_events
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_pending(&self, limit: i64) -> Result<Vec<DomainEvent>, sqlx::Error> {
        sqlx::query_as::<_, DomainEvent>(
            "SELECT id, topic, aggregate_type, aggregate_id, event_type, payload, publish_status, published_at, last_error, created_at, updated_at
             FROM domain_events
             WHERE publish_status IN ($1, $2)
             ORDER BY created_at ASC
             LIMIT $3",
        )
        .bind(EventPublishStatus::Pending.as_str())
        .bind(EventPublishStatus::Failed.as_str())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_published(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE domain_events
             SET publish_status = $2, published_at = $3, last_error = NULL, updated_at = $3
             WHERE id = $1",
        )
        .bind(id)
        .bind(EventPublishStatus::Published.as_str())
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(&self, id: Uuid, error_message: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE domain_events
             SET publish_status = $2, last_error = $3, updated_at = now()
             WHERE id = $1",
        )
        .bind(id)
        .bind(EventPublishStatus::Failed.as_str())
        .bind(error_message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
