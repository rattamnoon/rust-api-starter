use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::events::model::{DomainEvent, EventPublishStatus},
};

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct EventsQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub status: Option<String>,
    pub topic: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventResponse {
    pub id: Uuid,
    pub topic: String,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub publish_status: EventPublishStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventsListResponse {
    pub items: Vec<EventResponse>,
    pub page: i64,
    pub limit: i64,
}

impl TryFrom<DomainEvent> for EventResponse {
    type Error = AppError;

    fn try_from(value: DomainEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            topic: value.topic,
            aggregate_type: value.aggregate_type,
            aggregate_id: value.aggregate_id,
            event_type: value.event_type,
            payload: value.payload,
            publish_status: value.publish_status.parse().map_err(AppError::Internal)?,
            published_at: value.published_at,
            last_error: value.last_error,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}
