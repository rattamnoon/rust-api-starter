use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptStatus {
    Pending,
    Generated,
    Emailed,
    EmailFailed,
}

impl ReceiptStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Generated => "generated",
            Self::Emailed => "emailed",
            Self::EmailFailed => "email_failed",
        }
    }
}

impl fmt::Display for ReceiptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ReceiptStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "generated" => Ok(Self::Generated),
            "emailed" => Ok(Self::Emailed),
            "email_failed" => Ok(Self::EmailFailed),
            _ => Err(format!("unsupported receipt status: {value}")),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Receipt {
    pub id: Uuid,
    pub order_id: Uuid,
    pub receipt_number: String,
    pub status: String,
    pub upload_id: Option<Uuid>,
    pub issued_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Receipt {
    pub fn status(&self) -> Result<ReceiptStatus, String> {
        self.status.parse()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct EmailDelivery {
    pub id: Uuid,
    pub receipt_id: Uuid,
    pub template_key: String,
    pub recipient: String,
    pub subject: String,
    pub status: String,
    pub provider_message_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
