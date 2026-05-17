use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Draft,
    PendingPayment,
    Paid,
    PaymentFailed,
    Canceled,
    Fulfilled,
}

impl OrderStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingPayment => "pending_payment",
            Self::Paid => "paid",
            Self::PaymentFailed => "payment_failed",
            Self::Canceled => "canceled",
            Self::Fulfilled => "fulfilled",
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for OrderStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(Self::Draft),
            "pending_payment" => Ok(Self::PendingPayment),
            "paid" => Ok(Self::Paid),
            "payment_failed" => Ok(Self::PaymentFailed),
            "canceled" => Ok(Self::Canceled),
            "fulfilled" => Ok(Self::Fulfilled),
            _ => Err(format!("unsupported order status: {value}")),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    pub currency: String,
    pub stripe_checkout_session_id: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn status(&self) -> Result<OrderStatus, String> {
        self.status.parse()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub product_name_snapshot: String,
    pub sku_snapshot: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
    pub line_total_amount: i64,
    pub created_at: DateTime<Utc>,
}
