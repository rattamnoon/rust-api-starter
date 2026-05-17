use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::app_error::AppError,
    modules::products::model::{Currency, Product},
};

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProductRequest {
    #[validate(length(min = 2, message = "sku must be at least 2 characters"))]
    pub sku: String,
    #[validate(length(min = 2, message = "name must be at least 2 characters"))]
    pub name: String,
    pub description: Option<String>,
    pub price_amount: i64,
    pub currency: Currency,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProductRequest {
    #[validate(length(min = 2, message = "sku must be at least 2 characters"))]
    pub sku: Option<String>,
    #[validate(length(min = 2, message = "name must be at least 2 characters"))]
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub price_amount: Option<i64>,
    pub currency: Option<Currency>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ProductQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub active_only: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub price_amount: i64,
    pub currency: Currency,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProductsListResponse {
    pub items: Vec<ProductResponse>,
    pub page: i64,
    pub limit: i64,
}

impl ProductResponse {
    pub fn from_model(product: Product) -> Result<Self, AppError> {
        let currency = product.currency().map_err(AppError::Internal)?;
        Ok(Self {
            id: product.id,
            sku: product.sku,
            name: product.name,
            description: product.description,
            price_amount: product.price_amount,
            currency,
            is_active: product.is_active,
            created_at: product.created_at,
            updated_at: product.updated_at,
        })
    }
}
