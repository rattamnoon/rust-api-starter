use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::modules::{orders::dto::OrderResponse, payments::model::PaymentStatus};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckoutOrderRequest {
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CheckoutSessionResponse {
    pub order: OrderResponse,
    pub provider: String,
    pub checkout_session_id: String,
    pub checkout_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentWebhookAcceptedResponse {
    pub accepted: bool,
    pub workflow_id: Option<String>,
    pub status: Option<PaymentStatus>,
}
