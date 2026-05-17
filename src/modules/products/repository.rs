use sqlx::{PgPool, QueryBuilder, postgres::Postgres};
use uuid::Uuid;

use crate::modules::products::{
    dto::{CreateProductRequest, UpdateProductRequest},
    model::Product,
};

#[derive(Clone)]
pub struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: &CreateProductRequest) -> Result<Product, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "INSERT INTO products (sku, name, description, price_amount, currency, is_active)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, sku, name, description, price_amount, currency, is_active, created_at, updated_at",
        )
        .bind(&request.sku)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.price_amount)
        .bind(request.currency.as_str())
        .bind(request.is_active.unwrap_or(true))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list(
        &self,
        page: i64,
        limit: i64,
        active_only: Option<bool>,
    ) -> Result<Vec<Product>, sqlx::Error> {
        let offset = (page - 1).max(0) * limit;
        match active_only {
            Some(true) => {
                sqlx::query_as::<_, Product>(
                    "SELECT id, sku, name, description, price_amount, currency, is_active, created_at, updated_at
                     FROM products
                     WHERE is_active = true
                     ORDER BY created_at DESC
                     LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            _ => {
                sqlx::query_as::<_, Product>(
                    "SELECT id, sku, name, description, price_amount, currency, is_active, created_at, updated_at
                     FROM products
                     ORDER BY created_at DESC
                     LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, sku, name, description, price_amount, currency, is_active, created_at, updated_at
             FROM products WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_sku(&self, sku: &str) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, sku, name, description, price_amount, currency, is_active, created_at, updated_at
             FROM products WHERE sku = $1",
        )
        .bind(sku)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateProductRequest,
    ) -> Result<Option<Product>, sqlx::Error> {
        let has_changes = request.sku.is_some()
            || request.name.is_some()
            || request.description.is_some()
            || request.price_amount.is_some()
            || request.currency.is_some()
            || request.is_active.is_some();

        if !has_changes {
            return self.find_by_id(id).await;
        }

        let mut builder = QueryBuilder::<Postgres>::new("UPDATE products SET ");
        let mut separated = builder.separated(", ");
        if let Some(value) = request.sku {
            separated.push("sku = ").push_bind(value);
        }
        if let Some(value) = request.name {
            separated.push("name = ").push_bind(value);
        }
        if let Some(value) = request.description {
            separated.push("description = ").push_bind(value);
        }
        if let Some(value) = request.price_amount {
            separated.push("price_amount = ").push_bind(value);
        }
        if let Some(value) = request.currency {
            separated.push("currency = ").push_bind(value.as_str());
        }
        if let Some(value) = request.is_active {
            separated.push("is_active = ").push_bind(value);
        }
        let _ = separated;

        builder.push(" WHERE id = ").push_bind(id).push(
            " RETURNING id, sku, name, description, price_amount, currency, is_active, created_at, updated_at",
        );

        builder.build_query_as::<Product>().fetch_optional(&self.pool).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let affected = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(affected > 0)
    }
}
