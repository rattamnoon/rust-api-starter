use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::receipts::{
    model::{EmailDelivery, Receipt, ReceiptStatus},
};

#[derive(Clone)]
pub struct ReceiptRepository {
    pool: PgPool,
}

impl ReceiptRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, receipt_id: Uuid) -> Result<Option<Receipt>, sqlx::Error> {
        sqlx::query_as::<_, Receipt>(
            "SELECT id, order_id, receipt_number, status, upload_id, issued_at, created_at, updated_at
             FROM receipts WHERE id = $1",
        )
        .bind(receipt_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_order_id(&self, order_id: Uuid) -> Result<Option<Receipt>, sqlx::Error> {
        sqlx::query_as::<_, Receipt>(
            "SELECT id, order_id, receipt_number, status, upload_id, issued_at, created_at, updated_at
             FROM receipts WHERE order_id = $1",
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_or_get(
        &self,
        order_id: Uuid,
        receipt_number: &str,
    ) -> Result<Receipt, sqlx::Error> {
        sqlx::query_as::<_, Receipt>(
            "INSERT INTO receipts (order_id, receipt_number, status, issued_at)
             VALUES ($1, $2, $3, now())
             ON CONFLICT (order_id)
             DO UPDATE SET updated_at = now()
             RETURNING id, order_id, receipt_number, status, upload_id, issued_at, created_at, updated_at",
        )
        .bind(order_id)
        .bind(receipt_number)
        .bind(ReceiptStatus::Pending.as_str())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn mark_generated(
        &self,
        receipt_id: Uuid,
        upload_id: Uuid,
    ) -> Result<Receipt, sqlx::Error> {
        sqlx::query_as::<_, Receipt>(
            "UPDATE receipts
             SET status = $2, upload_id = $3, updated_at = now()
             WHERE id = $1
             RETURNING id, order_id, receipt_number, status, upload_id, issued_at, created_at, updated_at",
        )
        .bind(receipt_id)
        .bind(ReceiptStatus::Generated.as_str())
        .bind(upload_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn mark_delivery_status(
        &self,
        receipt_id: Uuid,
        status: ReceiptStatus,
    ) -> Result<Receipt, sqlx::Error> {
        sqlx::query_as::<_, Receipt>(
            "UPDATE receipts
             SET status = $2, updated_at = now()
             WHERE id = $1
             RETURNING id, order_id, receipt_number, status, upload_id, issued_at, created_at, updated_at",
        )
        .bind(receipt_id)
        .bind(status.as_str())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_email_delivery(
        &self,
        receipt_id: Uuid,
        recipient: &str,
        subject: &str,
    ) -> Result<EmailDelivery, sqlx::Error> {
        sqlx::query_as::<_, EmailDelivery>(
            "INSERT INTO email_deliveries (receipt_id, template_key, recipient, subject, status)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, receipt_id, template_key, recipient, subject, status, provider_message_id, error_message, created_at, updated_at",
        )
        .bind(receipt_id)
        .bind("receipt_email")
        .bind(recipient)
        .bind(subject)
        .bind("pending")
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_email_delivery(
        &self,
        delivery_id: Uuid,
        status: &str,
        provider_message_id: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<EmailDelivery, sqlx::Error> {
        sqlx::query_as::<_, EmailDelivery>(
            "UPDATE email_deliveries
             SET status = $2, provider_message_id = $3, error_message = $4, updated_at = now()
             WHERE id = $1
             RETURNING id, receipt_id, template_key, recipient, subject, status, provider_message_id, error_message, created_at, updated_at",
        )
        .bind(delivery_id)
        .bind(status)
        .bind(provider_message_id)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
    }
}
