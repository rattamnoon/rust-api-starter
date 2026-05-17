use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::{
        orders::model::{Order, OrderItem, OrderStatus},
        products::model::{Currency, Product},
    },
    shared::money::multiply_amount,
};

#[derive(Clone)]
pub struct OrderRepository {
    pool: PgPool,
}

pub struct NewOrderItem<'a> {
    pub product: &'a Product,
    pub quantity: i32,
}

impl OrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        currency: Currency,
        items: &[NewOrderItem<'_>],
    ) -> Result<Order, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let mut subtotal = 0_i64;

        for item in items {
            subtotal += multiply_amount(item.product.price_amount, item.quantity)
                .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
        }

        let order = sqlx::query_as::<_, Order>(
            "INSERT INTO orders (user_id, status, subtotal_amount, total_amount, currency)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, user_id, status, subtotal_amount, total_amount, currency, stripe_checkout_session_id, stripe_payment_intent_id, created_at, updated_at",
        )
        .bind(user_id)
        .bind(OrderStatus::Draft.as_str())
        .bind(subtotal)
        .bind(subtotal)
        .bind(currency.as_str())
        .fetch_one(&mut *transaction)
        .await?;

        for item in items {
            let line_total = multiply_amount(item.product.price_amount, item.quantity)
                .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
            sqlx::query(
                "INSERT INTO order_items (
                    order_id,
                    product_id,
                    product_name_snapshot,
                    sku_snapshot,
                    unit_price_amount,
                    quantity,
                    line_total_amount
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(order.id)
            .bind(item.product.id)
            .bind(&item.product.name)
            .bind(&item.product.sku)
            .bind(item.product.price_amount)
            .bind(item.quantity)
            .bind(line_total)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(order)
    }

    pub async fn find_by_id(&self, order_id: Uuid) -> Result<Option<Order>, sqlx::Error> {
        sqlx::query_as::<_, Order>(
            "SELECT id, user_id, status, subtotal_amount, total_amount, currency, stripe_checkout_session_id, stripe_payment_intent_id, created_at, updated_at
             FROM orders WHERE id = $1",
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_for_user(
        &self,
        user_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Order>, sqlx::Error> {
        let offset = (page - 1).max(0) * limit;
        sqlx::query_as::<_, Order>(
            "SELECT id, user_id, status, subtotal_amount, total_amount, currency, stripe_checkout_session_id, stripe_payment_intent_id, created_at, updated_at
             FROM orders
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_all(&self, page: i64, limit: i64) -> Result<Vec<Order>, sqlx::Error> {
        let offset = (page - 1).max(0) * limit;
        sqlx::query_as::<_, Order>(
            "SELECT id, user_id, status, subtotal_amount, total_amount, currency, stripe_checkout_session_id, stripe_payment_intent_id, created_at, updated_at
             FROM orders
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_items(&self, order_id: Uuid) -> Result<Vec<OrderItem>, sqlx::Error> {
        sqlx::query_as::<_, OrderItem>(
            "SELECT id, order_id, product_id, product_name_snapshot, sku_snapshot, unit_price_amount, quantity, line_total_amount, created_at
             FROM order_items WHERE order_id = $1 ORDER BY created_at ASC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_pending_payment(
        &self,
        order_id: Uuid,
        checkout_session_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE orders
             SET status = $2, stripe_checkout_session_id = $3, updated_at = now()
             WHERE id = $1",
        )
        .bind(order_id)
        .bind(OrderStatus::PendingPayment.as_str())
        .bind(checkout_session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_paid(&self, order_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE orders
             SET status = $2, updated_at = now()
             WHERE id = $1",
        )
        .bind(order_id)
        .bind(OrderStatus::Paid.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn attach_payment_intent(
        &self,
        order_id: Uuid,
        payment_intent_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE orders
             SET stripe_payment_intent_id = $2, updated_at = now()
             WHERE id = $1",
        )
        .bind(order_id)
        .bind(payment_intent_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
