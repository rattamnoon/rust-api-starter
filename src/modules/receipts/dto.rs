use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::receipts::model::{Receipt, ReceiptStatus},
};

#[derive(Debug, Serialize, ToSchema)]
pub struct ReceiptResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub receipt_number: String,
    pub status: ReceiptStatus,
    pub pdf_url: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ReceiptResponse {
    pub fn from_model(receipt: Receipt, pdf_url: Option<String>) -> Result<Self, AppError> {
        let status = receipt.status().map_err(AppError::Internal)?;
        Ok(Self {
            id: receipt.id,
            order_id: receipt.order_id,
            receipt_number: receipt.receipt_number,
            status,
            pdf_url,
            issued_at: receipt.issued_at,
            created_at: receipt.created_at,
            updated_at: receipt.updated_at,
        })
    }
}
