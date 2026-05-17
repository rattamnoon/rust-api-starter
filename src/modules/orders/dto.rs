use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::app_error::AppError,
    modules::{
        orders::model::{Order, OrderItem, OrderStatus},
        products::model::Currency,
    },
};

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateOrderItemRequest {
    pub product_id: Uuid,
    #[validate(range(min = 1, message = "quantity must be at least 1"))]
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateOrderRequest {
    #[validate(length(min = 1, message = "order must contain at least one item"))]
    pub items: Vec<CreateOrderItemRequest>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct OrderQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub sku: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
    pub line_total_amount: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: OrderStatus,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    pub currency: Currency,
    pub stripe_checkout_session_id: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub items: Vec<OrderItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrdersListResponse {
    pub items: Vec<OrderResponse>,
    pub page: i64,
    pub limit: i64,
}

impl OrderResponse {
    pub fn from_parts(order: Order, items: Vec<OrderItem>) -> Result<Self, AppError> {
        Ok(Self {
            id: order.id,
            user_id: order.user_id,
            status: order.status().map_err(AppError::Internal)?,
            subtotal_amount: order.subtotal_amount,
            total_amount: order.total_amount,
            currency: order.currency.parse().map_err(AppError::Internal)?,
            stripe_checkout_session_id: order.stripe_checkout_session_id,
            stripe_payment_intent_id: order.stripe_payment_intent_id,
            items: items.into_iter().map(Into::into).collect(),
            created_at: order.created_at,
            updated_at: order.updated_at,
        })
    }
}

impl From<OrderItem> for OrderItemResponse {
    fn from(value: OrderItem) -> Self {
        Self {
            id: value.id,
            product_id: value.product_id,
            product_name: value.product_name_snapshot,
            sku: value.sku_snapshot,
            unit_price_amount: value.unit_price_amount,
            quantity: value.quantity,
            line_total_amount: value.line_total_amount,
        }
    }
}
