use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    Thb,
    Usd,
}

impl Currency {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Thb => "thb",
            Self::Usd => "usd",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Currency {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "thb" => Ok(Self::Thb),
            "usd" => Ok(Self::Usd),
            _ => Err(format!("unsupported currency value: {value}")),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub price_amount: i64,
    pub currency: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Product {
    pub fn currency(&self) -> Result<Currency, String> {
        self.currency.parse()
    }
}
