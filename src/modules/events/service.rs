use serde_json::json;
use uuid::Uuid;

use crate::{
    config::settings::Settings,
    errors::app_error::AppError,
    modules::{
        events::{
            dto::{EventResponse, EventsListResponse, EventsQuery},
            repository::{EventRepository, NewDomainEvent},
        },
        orders::{model::Order, model::OrderItem},
        receipts::model::Receipt,
        users::model::User,
    },
    shared::{extractor::AuthenticatedUser, types::UserRole},
};

#[derive(Clone)]
pub struct EventService {
    repository: EventRepository,
    topic_users: String,
    topic_orders: String,
    topic_receipts: String,
}

impl EventService {
    pub fn new(settings: &Settings, repository: EventRepository) -> Self {
        Self {
            repository,
            topic_users: settings.kafka_topic_users.clone(),
            topic_orders: settings.kafka_topic_orders.clone(),
            topic_receipts: settings.kafka_topic_receipts.clone(),
        }
    }

    pub async fn record_user_registered(&self, user: &User) -> Result<String, AppError> {
        let event_type = "user.registered";
        self.repository
            .create(NewDomainEvent {
                topic: self.topic_users.clone(),
                aggregate_type: "user".to_string(),
                aggregate_id: user.id,
                event_type: event_type.to_string(),
                payload: json!({
                    "user_id": user.id,
                    "email": user.email,
                    "name": user.name,
                    "role": user.role,
                }),
            })
            .await?;
        Ok(event_type.to_string())
    }

    pub async fn record_order_created(
        &self,
        order: &Order,
        items: &[OrderItem],
    ) -> Result<String, AppError> {
        let event_type = "order.created";
        self.repository
            .create(NewDomainEvent {
                topic: self.topic_orders.clone(),
                aggregate_type: "order".to_string(),
                aggregate_id: order.id,
                event_type: event_type.to_string(),
                payload: json!({
                    "order_id": order.id,
                    "user_id": order.user_id,
                    "status": order.status,
                    "currency": order.currency,
                    "subtotal_amount": order.subtotal_amount,
                    "total_amount": order.total_amount,
                    "item_count": items.len(),
                }),
            })
            .await?;
        Ok(event_type.to_string())
    }

    pub async fn record_order_paid(
        &self,
        order: &Order,
        provider_payment_id: Option<&str>,
    ) -> Result<String, AppError> {
        let event_type = "order.paid";
        self.repository
            .create(NewDomainEvent {
                topic: self.topic_orders.clone(),
                aggregate_type: "order".to_string(),
                aggregate_id: order.id,
                event_type: event_type.to_string(),
                payload: json!({
                    "order_id": order.id,
                    "user_id": order.user_id,
                    "status": order.status,
                    "currency": order.currency,
                    "total_amount": order.total_amount,
                    "provider_payment_id": provider_payment_id,
                }),
            })
            .await?;
        Ok(event_type.to_string())
    }

    pub async fn record_receipt_generated(
        &self,
        receipt: &Receipt,
        pdf_url: Option<&str>,
    ) -> Result<String, AppError> {
        let event_type = "receipt.generated";
        self.repository
            .create(NewDomainEvent {
                topic: self.topic_receipts.clone(),
                aggregate_type: "receipt".to_string(),
                aggregate_id: receipt.id,
                event_type: event_type.to_string(),
                payload: json!({
                    "receipt_id": receipt.id,
                    "order_id": receipt.order_id,
                    "receipt_number": receipt.receipt_number,
                    "status": receipt.status,
                    "pdf_url": pdf_url,
                }),
            })
            .await?;
        Ok(event_type.to_string())
    }

    pub async fn record_receipt_emailed(
        &self,
        receipt: &Receipt,
        recipient: &str,
    ) -> Result<String, AppError> {
        let event_type = "receipt.emailed";
        self.repository
            .create(NewDomainEvent {
                topic: self.topic_receipts.clone(),
                aggregate_type: "receipt".to_string(),
                aggregate_id: receipt.id,
                event_type: event_type.to_string(),
                payload: json!({
                    "receipt_id": receipt.id,
                    "order_id": receipt.order_id,
                    "receipt_number": receipt.receipt_number,
                    "status": receipt.status,
                    "recipient": recipient,
                }),
            })
            .await?;
        Ok(event_type.to_string())
    }

    pub async fn list_events(
        &self,
        actor: &AuthenticatedUser,
        query: EventsQuery,
    ) -> Result<EventsListResponse, AppError> {
        ensure_admin(actor)?;
        let page = query.page.max(1);
        let limit = query.limit.clamp(1, 100);
        let items = self
            .repository
            .list(page, limit, query.status.as_deref(), query.topic.as_deref())
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<EventResponse>, AppError>>()?;
        Ok(EventsListResponse { items, page, limit })
    }

    pub async fn get_event(
        &self,
        actor: &AuthenticatedUser,
        event_id: Uuid,
    ) -> Result<EventResponse, AppError> {
        ensure_admin(actor)?;
        let event = self
            .repository
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| AppError::NotFound("event was not found".into()))?;
        event.try_into()
    }
}

fn ensure_admin(actor: &AuthenticatedUser) -> Result<(), AppError> {
    if actor.role == UserRole::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin access is required".into()))
    }
}
