use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::modules::payments::{model::Payment, model::WorkflowRun};

#[derive(Clone)]
pub struct PaymentRepository {
    pool: PgPool,
}

pub struct NewPayment {
    pub order_id: Uuid,
    pub provider: String,
    pub provider_payment_id: Option<String>,
    pub provider_session_id: Option<String>,
    pub status: String,
    pub amount: i64,
    pub currency: String,
    pub raw_payload: Value,
}

impl PaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_checkout_payment(
        &self,
        input: NewPayment,
    ) -> Result<Payment, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            "INSERT INTO payments (
                order_id, provider, provider_payment_id, provider_session_id, status, amount, currency, raw_payload
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (order_id, provider, provider_session_id)
             DO UPDATE SET
                provider_payment_id = excluded.provider_payment_id,
                status = excluded.status,
                amount = excluded.amount,
                currency = excluded.currency,
                raw_payload = excluded.raw_payload,
                updated_at = now()
             RETURNING id, order_id, provider, provider_payment_id, provider_session_id, status, amount, currency, raw_payload, created_at, updated_at",
        )
        .bind(input.order_id)
        .bind(input.provider)
        .bind(input.provider_payment_id)
        .bind(input.provider_session_id)
        .bind(input.status)
        .bind(input.amount)
        .bind(input.currency)
        .bind(Json(input.raw_payload))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_latest_by_order(&self, order_id: Uuid) -> Result<Option<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            "SELECT id, order_id, provider, provider_payment_id, provider_session_id, status, amount, currency, raw_payload, created_at, updated_at
             FROM payments
             WHERE order_id = $1
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn register_webhook_event(
        &self,
        event_id: &str,
        event_type: &str,
        payload: Value,
    ) -> Result<bool, sqlx::Error> {
        let affected = sqlx::query(
            "INSERT INTO payment_webhook_events (provider_event_id, event_type, payload)
             VALUES ($1, $2, $3)
             ON CONFLICT (provider_event_id) DO NOTHING",
        )
        .bind(event_id)
        .bind(event_type)
        .bind(Json(payload))
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(affected > 0)
    }

    pub async fn upsert_workflow_run(
        &self,
        order_id: Uuid,
        workflow_id: &str,
        namespace: &str,
        task_queue: &str,
        status: &str,
    ) -> Result<WorkflowRun, sqlx::Error> {
        sqlx::query_as::<_, WorkflowRun>(
            "INSERT INTO workflow_runs (order_id, workflow_id, namespace, task_queue, status, started_at)
             VALUES ($1, $2, $3, $4, $5, now())
             ON CONFLICT (order_id)
             DO UPDATE SET workflow_id = excluded.workflow_id, namespace = excluded.namespace, task_queue = excluded.task_queue, status = excluded.status, started_at = now(), updated_at = now(), last_error = null
             RETURNING id, order_id, workflow_id, namespace, task_queue, status, last_error, started_at, finished_at, created_at, updated_at",
        )
        .bind(order_id)
        .bind(workflow_id)
        .bind(namespace)
        .bind(task_queue)
        .bind(status)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_workflow_run_status(
        &self,
        order_id: Uuid,
        status: &str,
        last_error: Option<String>,
        finished_at: Option<DateTime<Utc>>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workflow_runs
             SET status = $2, last_error = $3, finished_at = $4, updated_at = now()
             WHERE order_id = $1",
        )
        .bind(order_id)
        .bind(status)
        .bind(last_error)
        .bind(finished_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
