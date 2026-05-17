use actix_web::{HttpResponse, web};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::app_error::AppError,
    modules::orders::dto::{CreateOrderRequest, OrderQuery},
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    post,
    path = "/api/v1/orders",
    tag = "Orders",
    security(("bearer_auth" = [])),
    request_body = CreateOrderRequest,
    responses(
        (status = 201, body = crate::modules::orders::dto::OrderResponse),
        (status = 400, body = ErrorResponseBody),
        (status = 401, body = ErrorResponseBody)
    )
)]
pub async fn create_order(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    request: web::Json<CreateOrderRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
    let response = state
        .order_service
        .create_order(&current_user.0, request.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders",
    tag = "Orders",
    security(("bearer_auth" = [])),
    params(OrderQuery),
    responses((status = 200, body = crate::modules::orders::dto::OrdersListResponse))
)]
pub async fn list_orders(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    query: web::Query<OrderQuery>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .order_service
        .list_orders(&current_user.0, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders/{id}",
    tag = "Orders",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Order ID")),
    responses(
        (status = 200, body = crate::modules::orders::dto::OrderResponse),
        (status = 403, body = ErrorResponseBody),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn get_order(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    order_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .order_service
        .get_order(&current_user.0, order_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}
