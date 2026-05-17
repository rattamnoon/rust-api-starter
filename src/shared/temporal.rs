use chrono::Utc;
use uuid::Uuid;

use crate::{
    config::settings::Settings,
    errors::app_error::AppError,
    modules::{
        orders::{model::OrderStatus, repository::OrderRepository},
        payments::repository::PaymentRepository,
        receipts::service::ReceiptService,
    },
};

#[derive(Clone)]
pub struct TemporalCommerceService {
    order_repository: OrderRepository,
    payment_repository: PaymentRepository,
    receipt_service: ReceiptService,
    namespace: String,
    task_queue: String,
}

impl TemporalCommerceService {
    pub fn new(
        settings: &Settings,
        order_repository: OrderRepository,
        payment_repository: PaymentRepository,
        receipt_service: ReceiptService,
    ) -> Self {
        Self {
            order_repository,
            payment_repository,
            receipt_service,
            namespace: settings.temporal_namespace.clone(),
            task_queue: settings.temporal_task_queue.clone(),
        }
    }

    pub async fn process_paid_order(
        &self,
        order_id: Uuid,
        external_event_id: &str,
    ) -> Result<String, AppError> {
        let workflow_id = format!("order-{order_id}");
        self.payment_repository
            .upsert_workflow_run(order_id, &workflow_id, &self.namespace, &self.task_queue, "running")
            .await?;

        let order = self
            .order_repository
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("order was not found".into()))?;
        let status = order.status().map_err(AppError::Internal)?;

        if !matches!(status, OrderStatus::PendingPayment | OrderStatus::Paid | OrderStatus::Fulfilled)
        {
            return Err(AppError::BadRequest(
                "only pending or paid orders can enter the receipt workflow".into(),
            ));
        }

        self.order_repository.mark_paid(order_id).await?;
        let receipt = self
            .receipt_service
            .generate_and_send_receipt(order_id, external_event_id)
            .await;

        match receipt {
            Ok(_) => {
                self.payment_repository
                    .update_workflow_run_status(order_id, "completed", None, Some(Utc::now()))
                    .await?;
                Ok(workflow_id)
            }
            Err(error) => {
                self.payment_repository
                    .update_workflow_run_status(
                        order_id,
                        "failed",
                        Some(error.to_string()),
                        Some(Utc::now()),
                    )
                    .await?;
                Err(error)
            }
        }
    }
}
