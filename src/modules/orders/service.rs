use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::{
        events::service::EventService,
        orders::{
            dto::{CreateOrderRequest, OrderQuery, OrderResponse, OrdersListResponse},
            repository::{NewOrderItem, OrderRepository},
        },
        products::repository::ProductRepository,
    },
    shared::{extractor::AuthenticatedUser, types::UserRole},
};

#[derive(Clone)]
pub struct OrderService {
    order_repository: OrderRepository,
    product_repository: ProductRepository,
    event_service: EventService,
}

impl OrderService {
    pub fn new(
        order_repository: OrderRepository,
        product_repository: ProductRepository,
        event_service: EventService,
    ) -> Self {
        Self {
            order_repository,
            product_repository,
            event_service,
        }
    }

    pub async fn create_order(
        &self,
        actor: &AuthenticatedUser,
        request: CreateOrderRequest,
    ) -> Result<OrderResponse, AppError> {
        let mut products = Vec::with_capacity(request.items.len());
        for item in &request.items {
            let product = self
                .product_repository
                .find_by_id(item.product_id)
                .await?
                .ok_or_else(|| AppError::NotFound("product was not found".into()))?;
            if !product.is_active {
                return Err(AppError::BadRequest(format!(
                    "product `{}` is inactive",
                    product.name
                )));
            }
            products.push(product);
        }

        let currency = products
            .first()
            .ok_or_else(|| AppError::BadRequest("order must include at least one item".into()))?
            .currency()
            .map_err(AppError::Internal)?;

        if products
            .iter()
            .any(|product| product.currency != currency.as_str())
        {
            return Err(AppError::BadRequest(
                "all order items must use the same currency".into(),
            ));
        }

        let new_items = request
            .items
            .iter()
            .zip(products.iter())
            .map(|(request_item, product)| NewOrderItem {
                product,
                quantity: request_item.quantity,
            })
            .collect::<Vec<_>>();

        let order = self
            .order_repository
            .create(actor.user_id, currency, &new_items)
            .await?;
        let items = self.order_repository.list_items(order.id).await?;
        let _ = self
            .event_service
            .record_order_created(&order, &items)
            .await?;
        OrderResponse::from_parts(order, items)
    }

    pub async fn list_orders(
        &self,
        actor: &AuthenticatedUser,
        query: OrderQuery,
    ) -> Result<OrdersListResponse, AppError> {
        let page = query.page.max(1);
        let limit = query.limit.clamp(1, 100);
        let orders = if actor.role == UserRole::Admin {
            self.order_repository.list_all(page, limit).await?
        } else {
            self.order_repository
                .list_for_user(actor.user_id, page, limit)
                .await?
        };

        let mut items = Vec::with_capacity(orders.len());
        for order in orders {
            let order_items = self.order_repository.list_items(order.id).await?;
            items.push(OrderResponse::from_parts(order, order_items)?);
        }

        Ok(OrdersListResponse { items, page, limit })
    }

    pub async fn get_order(
        &self,
        actor: &AuthenticatedUser,
        order_id: Uuid,
    ) -> Result<OrderResponse, AppError> {
        let order = self
            .order_repository
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("order was not found".into()))?;
        ensure_owner_or_admin(actor, &order)?;
        let items = self.order_repository.list_items(order.id).await?;
        OrderResponse::from_parts(order, items)
    }
}

fn ensure_owner_or_admin(
    actor: &AuthenticatedUser,
    order: &crate::modules::orders::model::Order,
) -> Result<(), AppError> {
    if actor.role == UserRole::Admin || actor.user_id == order.user_id {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "you can only access your own orders".into(),
        ))
    }
}
