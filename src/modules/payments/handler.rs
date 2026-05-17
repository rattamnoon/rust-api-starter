use actix_web::{HttpRequest, HttpResponse, web};
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::payments::dto::CheckoutOrderRequest,
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    post,
    path = "/api/v1/orders/{id}/checkout",
    tag = "Payments",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Order ID")),
    request_body = CheckoutOrderRequest,
    responses(
        (status = 200, body = crate::modules::payments::dto::CheckoutSessionResponse),
        (status = 400, body = ErrorResponseBody),
        (status = 403, body = ErrorResponseBody),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn create_checkout_session(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    order_id: web::Path<Uuid>,
    request: web::Json<CheckoutOrderRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .payment_service
        .create_checkout_session(&current_user.0, order_id.into_inner(), request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/payments/webhooks/stripe",
    tag = "Payments",
    request_body(
        content = String,
        content_type = "application/json"
    ),
    responses(
        (status = 200, body = crate::modules::payments::dto::PaymentWebhookAcceptedResponse),
        (status = 401, body = ErrorResponseBody)
    )
)]
pub async fn stripe_webhook(
    state: web::Data<AppState>,
    request: HttpRequest,
    payload: web::Bytes,
) -> Result<HttpResponse, AppError> {
    let signature = request
        .headers()
        .get("Stripe-Signature")
        .and_then(|value| value.to_str().ok());
    let response = state
        .payment_service
        .handle_stripe_webhook(signature, &payload)
        .await?;
    Ok(HttpResponse::Ok().json(response))
}
